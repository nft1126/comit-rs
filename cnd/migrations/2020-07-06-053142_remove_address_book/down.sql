-- This file should undo anything in `up.sql`

CREATE TABLE address_book
(
    id                          INTEGER NOT NULL PRIMARY KEY,
    peer_id                     NOT NULL,
    multi_address                NOT NULL
);
