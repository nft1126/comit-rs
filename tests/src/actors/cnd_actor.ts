import {
    ActionKind,
    ActiveSwapsEntity,
    MarketEntity,
    OpenOrdersEntity,
    OrderEntity,
    Position,
    RedeemEvent,
    SwapEntity,
    SwapEventKind,
} from "../cnd_client/payload";
import { Logger } from "log4js";
import { CndInstance } from "../environment/cnd_instance";
import { sleep } from "../utils";
import { DumpState, GetListenAddress, GetPeerId, Role, Stoppable } from "./index";
import { Wallets } from "../wallets";
import pTimeout from "p-timeout";
import { AxiosResponse } from "axios";
import CndClient from "../cnd_client";
import { Swap } from "../swap";
import { Entity } from "../cnd_client/siren";
import { HarnessGlobal } from "../environment";
import { BalanceAsserter, Erc20BalanceAsserter, OnChainBitcoinBalanceAsserter } from "./balance_asserter";
import { parseFixed } from "@ethersproject/bignumber";

declare var global: HarnessGlobal;

/**
 * An actor that uses cnd to perform to participate in the COMIT network.
 *
 * Although in reality instance of cnd can handle multiple swaps in different roles at the same time, the test framework limits an instance to one specific role.
 */
export class CndActor implements Stoppable, DumpState, GetListenAddress, GetPeerId {
    readonly cnd: CndClient;
    public swap: Swap;

    private alphaBalance: BalanceAsserter;
    private betaBalance: BalanceAsserter;
    private mostRecentOrderHref: string;

    public constructor(
        public readonly logger: Logger,
        public readonly cndInstance: CndInstance,
        public readonly wallets: Wallets,
        public readonly role: Role,
    ) {
        logger.info(
            "Created new actor in role",
            role,
            "with config",
            cndInstance.config,
        );
        const socket = cndInstance.config.http_api.socket;
        this.cnd = new CndClient(`http://${socket}`);
    }

    async getPeerId(): Promise<string> {
        return this.cnd.getPeerId();
    }

    async getListenAddress(): Promise<string> {
        const listenAddresses = await this.cnd.getPeerListenAddresses();

        return listenAddresses[0];
    }

    public async connect<O extends GetListenAddress & GetPeerId>(other: O) {
        const listenAddress = await other.getListenAddress();
        const otherPeerId = await other.getPeerId();

        this.logger.info("Connecting to", otherPeerId, "on", listenAddress);

        await this.cnd.dial(listenAddress);

        await this.pollUntilConnectedTo(otherPeerId);

        this.logger.info(
            "Successfully connected to",
            otherPeerId,
            "on",
            listenAddress,
        );
    }

    public async makeBtcDaiOrder(
        position: Position,
        quantity: string,
        price: string,
    ): Promise<string> {
        const sats = BigInt(parseFixed(quantity, 8).toString());
        const weiPerBtc = BigInt(parseFixed(price, 18).toString());

        const satsPerBtc = 100000000n;
        const weiPerSat = weiPerBtc / satsPerBtc;
        const dai = sats * weiPerSat;

        switch (position) {
            case Position.Buy: {
                switch (this.role) {
                    case "Alice": {
                        this.alphaBalance = await Erc20BalanceAsserter.newInstance(
                            this.wallets.ethereum,
                            dai,
                            global.tokenContract,
                        );
                        this.betaBalance = await OnChainBitcoinBalanceAsserter.newInstance(
                            this.wallets.bitcoin,
                            sats,
                        );
                        break;
                    }
                    case "Bob": {
                        this.alphaBalance = await OnChainBitcoinBalanceAsserter.newInstance(
                            this.wallets.bitcoin,
                            sats,
                        );
                        this.betaBalance = await Erc20BalanceAsserter.newInstance(
                            this.wallets.ethereum,
                            dai,
                            global.tokenContract,
                        );
                        break;
                    }
                }
                break;
            }
            case Position.Sell: {
                switch (this.role) {
                    case "Alice": {
                        this.alphaBalance = await OnChainBitcoinBalanceAsserter.newInstance(
                            this.wallets.bitcoin,
                            sats,
                        );
                        this.betaBalance = await Erc20BalanceAsserter.newInstance(
                            this.wallets.ethereum,
                            dai,
                            global.tokenContract,
                        );
                        break;
                    }
                    case "Bob": {
                        this.alphaBalance = await Erc20BalanceAsserter.newInstance(
                            this.wallets.ethereum,
                            dai,
                            global.tokenContract,
                        );
                        this.betaBalance = await OnChainBitcoinBalanceAsserter.newInstance(
                            this.wallets.bitcoin,
                            sats,
                        );
                        break;
                    }
                }
                break;
            }
        }

        this.mostRecentOrderHref = await this.cnd.createBtcDaiOrder({
            position,
            quantity: sats,
            price: weiPerSat,
            swap: {
                role: this.role,
                bitcoin_address: await this.wallets.bitcoin.getAddress(),
                ethereum_address: this.wallets.ethereum.getAccount(),
            },
        });

        return this.mostRecentOrderHref;
    }

    public async getBtcDaiMarket(): Promise<MarketEntity> {
        return this.cnd
            .fetch<MarketEntity>("/markets/BTC-DAI")
            .then((r) => r.data);
    }

    public async fetchOrder(href: string): Promise<OrderEntity> {
        const response = await this.cnd.fetch<OrderEntity>(href);

        return response.data;
    }

    public async listOpenOrders(): Promise<OpenOrdersEntity> {
        const response = await this.cnd.fetch<OpenOrdersEntity>("/orders");

        return response.data;
    }

    public async executeSirenAction<T>(
        entity: Entity,
        actionName: string,
    ): Promise<AxiosResponse<T>> {
        const action = entity.actions.find(
            (action) => action.name === actionName,
        );

        if (!action) {
            throw new Error(`Action ${actionName} is not present`);
        }

        return this.cnd.executeSirenAction(action);
    }

    public cndHttpApiUrl() {
        const socket = this.cndInstance.config.http_api.socket;
        return `http://${socket}`;
    }

    public async pollCndUntil<T>(
        location: string,
        predicate: (body: T) => boolean,
    ): Promise<T> {
        const poller = async () => {
            let response = await this.cnd.fetch<T>(location);

            while (!predicate(response.data)) {
                response = await this.cnd.fetch<T>(location);
                await sleep(500);
            }

            return response.data;
        };

        return pTimeout(
            poller(),
            10_000,
            new Error(
                `response from ${location} did not satisfy predicate after 10 seconds`,
            ),
        );
    }

    public async assertAndExecuteNextAction(expectedActionName: ActionKind) {
        if (!this.swap) {
            throw new Error("Cannot do anything on non-existent swap");
        }

        const { action, transaction } = await pTimeout(
            (async () => {
                while (true) {
                    const action = await this.swap.nextAction();

                    if (action && action.name === expectedActionName) {
                        const transaction = await action.execute();
                        return { action, transaction };
                    }

                    await sleep(1000);
                }
            })(),
            20 * 1000,
            `action '${expectedActionName}' not found`,
        );

        this.logger.debug(
            "%s done on swap %s in %s",
            action.name,
            this.swap.self,
            transaction,
        );

        const swapProperties = await this.getSwapResponse().then(
            (e) => e.properties,
        );
        const event = nextExpectedEvent(
            swapProperties.role,
            expectedActionName,
            swapProperties.alpha.protocol,
            swapProperties.beta.protocol,
        );

        if (event === null) {
            return;
        }

        await pTimeout(
            (async () => {
                while (true) {
                    const swapEntity = await this.getSwapResponse();

                    if (
                        swapEntity.properties.events
                            .map((e) => e.name)
                            .includes(event)
                    ) {
                        return;
                    }

                    await sleep(500);
                }
            })(),
            30_000,
            `event '${event}' expected but never found`,
        );
    }

    public async getSwapResponse(): Promise<SwapEntity> {
        return this.cnd
            .fetch<SwapEntity>(this.swap.self)
            .then((response) => response.data);
    }

    public async assertBalancesAfterSwap() {
        this.logger.debug("Checking if swap @ %s swapped", this.swap.self);

        switch (this.role) {
            case "Alice": {
                await this.alphaBalance.assertSpent();
                await this.betaBalance.assertReceived();
                break;
            }
            case "Bob": {
                await this.alphaBalance.assertReceived();
                await this.betaBalance.assertSpent();
                break;
            }
        }
    }

    public async assertBalancesAfterRefund() {
        this.logger.debug("Checking if swap @ %s was refunded", this.swap.self);

        switch (this.role) {
            case "Alice": {
                await this.alphaBalance.assertRefunded();
                await this.betaBalance.assertNothingReceived();
                break;
            }
            case "Bob": {
                await this.alphaBalance.assertNothingReceived();
                await this.betaBalance.assertRefunded();
                break;
            }
        }
    }

    public async assertOrderClosed() {
        const order = await this.cnd.fetch<OrderEntity>(
            this.mostRecentOrderHref,
        );

        expect(order.data.properties.state).toMatchObject({
            closed: order.data.properties.quantity.value,
            open: "0",
            settling: "0",
            failed: "0",
            cancelled: "0",
        });
    }

    public async assertSwapInactive() {
        const activeSwaps = await this.cnd.fetch<ActiveSwapsEntity>("/swaps");

        expect(activeSwaps.data.entities).toHaveLength(0);
    }

    /**
     * Manage cnd instance
     */

    public async start() {
        await this.cndInstance.start();
    }

    public async stop() {
        this.logger.debug("Stopping actor");
        this.cndInstance.stop();
    }

    public async restart() {
        await this.stop();
        await this.start();
    }

    public async dumpState() {
        this.logger.debug("dumping current state");

        if (this.swap) {
            const swapResponse = await this.getSwapResponse();

            this.logger.debug("swap details: ", JSON.stringify(swapResponse));
        }
    }

    public async waitForSwap(): Promise<void> {
        const response = await this.pollCndUntil<Entity>(
            "/swaps",
            (body) => body.entities.length > 0,
        );

        this.swap = new Swap(
            this.cnd,
            response.entities[0].href,
            new Wallets({
                ethereum: this.wallets.ethereum,
                bitcoin: this.wallets.bitcoin,
            }),
        );
    }

    public async waitUntilSwapped() {
        type RedeemEventKind = RedeemEvent["name"];

        await this.pollCndUntil<SwapEntity>(this.swap.self, (body) => {
            const eventNames = body.properties.events.map((e) => e.name);

            const alphaRedeemed = `${body.properties.alpha.protocol}_redeemed` as RedeemEventKind;
            const betaRedeemed = `${body.properties.beta.protocol}_redeemed` as RedeemEventKind;

            return (
                eventNames.includes(alphaRedeemed)
                && eventNames.includes(betaRedeemed)
            );
        });
    }

    public async pollUntilConnectedTo(peer: string) {
        interface Peers {
            peers: Peer[];
        }

        interface Peer {
            id: string;
            // these are multi-addresses
            endpoints: string[];
        }

        await this.pollCndUntil<Peers>(
            "/peers",
            (peers) =>
                peers.peers.findIndex((candidate) => candidate.id === peer)
                    !== -1,
        );
    }
}

/**
 * Computes the event that we are expecting to see.
 */
function nextExpectedEvent(
    role: "Alice" | "Bob",
    action: ActionKind,
    alphaProtocol: "hbit" | "herc20",
    betaProtocol: "hbit" | "herc20",
): SwapEventKind {
    switch (action) {
        // "deploy" can only mean we are waiting for "herc20_deployed"
        case "deploy": {
            return "herc20_deployed";
        }

        // Alice is always funding and refunding on the alpha ledger, likewise Bob on the beta ledger

        case "fund":
        case "refund": {
            switch (role) {
                case "Alice": {
                    // @ts-ignore: Sad that TypeScript can't infer that.
                    return `${alphaProtocol}_${action}ed`;
                }
                case "Bob": {
                    // @ts-ignore: Sad that TypeScript can't infer that.
                    return `${betaProtocol}_${action}ed`;
                }
                default:
                    throw new Error(
                        `Who is ${role}? We expected either Alice or Bob!`,
                    );
            }
        }
        // Alice is always redeeming on the beta ledger, likewise Bob on the alpha ledger
        case "redeem": {
            switch (role) {
                case "Alice": {
                    // @ts-ignore: Sad that TypeScript can't infer that.
                    return `${betaProtocol}_${action}ed`;
                }
                case "Bob": {
                    // @ts-ignore: Sad that TypeScript can't infer that.
                    return `${alphaProtocol}_${action}ed`;
                }
                default:
                    throw new Error(
                        `Who is ${role}? We expected either Alice or Bob!`,
                    );
            }
        }
    }
}
