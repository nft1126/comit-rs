/**
 * Payloads for the `/swaps/` REST API.
 */

/*
 * The payload for POST requests to create a swap on the cnd REST API.
 */
import { Action, EmbeddedLinkSubEntity, EmbeddedRepresentationSubEntity, Entity, Link } from "./siren";

export interface Peer {
    peer_id: string;
    address_hint?: string;
}

/**
 * The payload returned when fetching one swap on the `/swaps/:id` endpoint
 */
export interface SwapEntity extends Entity {
    properties: SwapProperties;
    actions: SwapAction[];
    /**
     * links for this swap, contains a self reference
     */
    links: Link[];
}

export interface ActiveSwapsEntity extends Entity {
    entities: EmbeddedLinkSubEntity[];
}

/**
 * The properties of a swap
 */
export interface SwapProperties {
    /**
     * The role in which you are participating in this swap.
     */
    role: "Alice" | "Bob";
    /**
     * The linear sequence of events related to this swap as observed by cnd.
     */
    events: SwapEvent[];
    alpha: LockProtocol;
    beta: LockProtocol;
}

export type LockProtocol = HbitProtocol | Herc20Protocol;

export interface HbitProtocol {
    protocol: "hbit";
    asset: Amount;
}

export interface Herc20Protocol {
    protocol: "herc20";
    asset: Amount;
}

export type FundEvent = HbitFundedEvent | Herc20FundedEvent;
export type RefundEvent = HbitRefundedEvent | Herc20RefundedEvent;
export type RedeemEvent = HbitRedeemedEvent | Herc20RedeemedEvent;

export type SwapEvent =
    | FundEvent
    | RefundEvent
    | RedeemEvent
    | Herc20DeployedEvent;

export type SwapEventKind = SwapEvent["name"]; // Oh yeah, type system magic baby!

export interface HbitFundedEvent {
    name: "hbit_funded";
    tx: string;
}

export interface HbitRedeemedEvent {
    name: "hbit_redeemed";
    tx: string;
}

export interface HbitRefundedEvent {
    name: "hbit_refunded";
    tx: string;
}

export interface Herc20DeployedEvent {
    name: "herc20_deployed";
    tx: string;
}

export interface Herc20FundedEvent {
    name: "herc20_funded";
    tx: string;
}

export interface Herc20RedeemedEvent {
    name: "herc20_redeemed";
    tx: string;
}

export interface Herc20RefundedEvent {
    name: "herc20_refunded";
    tx: string;
}

/**
 * The possible steps needed on each side of the swap for its execution.
 *
 * Not all steps are needed for all protocols and ledgers.
 * E.g. for Han Bitcoin the steps are: fund, redeem (or refund)
 */
export type ActionKind = "fund" | "deploy" | "redeem" | "refund";

/**
 * An action that is available for the given swap.
 */
export interface SwapAction extends Action {
    name: ActionKind;
}

export enum Position {
    Buy = "buy",
    Sell = "sell",
}

export interface MarketEntity extends Entity {
    entities: MarketItemEntity[];
}

export interface MarketItemEntity extends EmbeddedRepresentationSubEntity {
    properties: MarketItemProperties;
}

export interface MarketItemProperties {
    id: string;
    position: Position;
    quantity: Amount;
    price: Amount;
    ours: boolean;
    maker: string;
}

export interface OrderEntity extends Entity {
    properties: OrderProperties;
}

export interface OrderProperties {
    id: string;
    position: Position;
    quantity: Amount;
    price: Amount;
    state: {
        open: string;
        closed: string;
        settling: string;
        failed: string;
        cancelled: string;
    };
}

export interface Amount {
    currency: Currency;
    value: string;
    decimals: number;
}

export enum Currency {
    BTC = "BTC",
    DAI = "DAI",
}

export interface OpenOrderEntity extends EmbeddedRepresentationSubEntity {
    properties: OrderProperties;
    rel: ["item"];
}

export interface OpenOrdersEntity extends Entity {
    entities: OpenOrderEntity[];
}

export interface CreateBtcDaiOrderPayload {
    position: Position;
    quantity: bigint;
    price: bigint;
    swap: {
        role: string;
        bitcoin_address: string;
        ethereum_address: string;
    };
}

export interface GetInfoResponse {
    id: string;
    listen_addresses: string[]; // multiaddresses
}
