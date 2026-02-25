/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/chaintrace.json`.
 */
export type ageis = {
  "address": "4fRvr5yrDNTqnSXv8yFb9CSj3MwnYuade8UUmgb8cg3H",
  "metadata": {
    "name": "chaintrace",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Created with Anchor"
  },
  "instructions": [
    {
      "name": "initializeDelivery",
      "docs": [
        "Record NFC tap by farmer during packaging — creates the DeliveryTrace PDA.",
        "",
        "Edge cases handled:",
        "- Requires ProductTrace to already exist (seeds derive same order_id).",
        "If ProductTrace PDA is absent the ix fails naturally because the",
        "`product_trace` account constraint won't resolve.",
        "- Double-initialization prevented: `init` on DeliveryTrace PDA will",
        "fail if it already exists, returning AccountAlreadyInUse.",
        "We surface a cleaner error via a manual check.",
        "- All DB foreign-key IDs validated for non-empty / length."
      ],
      "discriminator": [
        44,
        98,
        240,
        48,
        68,
        155,
        7,
        171
      ],
      "accounts": [
        {
          "name": "serverAuthority",
          "writable": true,
          "signer": true
        },
        {
          "name": "productTrace",
          "docs": [
            "Must already exist — validated by PDA seed match."
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  100,
                  117,
                  99,
                  116
                ]
              },
              {
                "kind": "arg",
                "path": "args.order_id"
              }
            ]
          }
        },
        {
          "name": "deliveryTrace",
          "docs": [
            "DeliveryTrace PDA — seeds: [\"delivery\", order_id]."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  101,
                  108,
                  105,
                  118,
                  101,
                  114,
                  121
                ]
              },
              {
                "kind": "arg",
                "path": "args.order_id"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "args",
          "type": {
            "defined": {
              "name": "initDeliveryArgs"
            }
          }
        }
      ]
    },
    {
      "name": "mintProductNft",
      "docs": [
        "Mint a product NFT and create the ProductTrace PDA.",
        "",
        "Called by the server when a user places an order (or when a farmer",
        "confirms a product listing).  The server must have already uploaded",
        "metadata to IPFS and resolved the CID before calling this.",
        "",
        "Edge cases handled:",
        "- Empty / oversized string fields rejected.",
        "- Idempotency: if a ProductTrace PDA already exists for the same",
        "order_id the ix will fail at account init time (Anchor prevents",
        "double-init on a PDA with `init`).  No extra check needed.",
        "- NFT supply is hard-capped at 1 via MasterEdition (max_supply = 0",
        "means \"unlimited prints\"; we pass Some(0) which means 0 prints",
        "allowed → effectively a 1/1)."
      ],
      "discriminator": [
        135,
        140,
        185,
        197,
        76,
        99,
        101,
        140
      ],
      "accounts": [
        {
          "name": "serverAuthority",
          "docs": [
            "The server keypair — must sign every tx."
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "mint",
          "docs": [
            "Fresh mint account created by this ix."
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "tokenAccount",
          "docs": [
            "ATA for the server to hold the minted token."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "serverAuthority"
              },
              {
                "kind": "const",
                "value": [
                  6,
                  221,
                  246,
                  225,
                  215,
                  101,
                  161,
                  147,
                  217,
                  203,
                  225,
                  70,
                  206,
                  235,
                  121,
                  172,
                  28,
                  180,
                  133,
                  237,
                  95,
                  91,
                  55,
                  145,
                  58,
                  140,
                  245,
                  133,
                  126,
                  255,
                  0,
                  169
                ]
              },
              {
                "kind": "account",
                "path": "mint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "productTrace",
          "docs": [
            "ProductTrace PDA — seeds: [\"product\", order_id]."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  100,
                  117,
                  99,
                  116
                ]
              },
              {
                "kind": "arg",
                "path": "args.order_id"
              }
            ]
          }
        },
        {
          "name": "metadata",
          "writable": true
        },
        {
          "name": "masterEdition",
          "writable": true
        },
        {
          "name": "tokenMetadataProgram",
          "address": "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "associatedTokenProgram",
          "address": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "rent",
          "address": "SysvarRent111111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "args",
          "type": {
            "defined": {
              "name": "mintProductArgs"
            }
          }
        }
      ]
    },
    {
      "name": "updateDeliveryStatus",
      "docs": [
        "Advance delivery status and record timestamp.",
        "",
        "Edge cases handled:",
        "- Only forward transitions allowed (status can only increase).",
        "Prevents replayed or out-of-order server calls from corrupting state.",
        "- Skipping a status is NOT allowed (e.g., 0 → 2 is rejected).",
        "Every step must be recorded so the trace is complete.",
        "- Attempting to move past DELIVERED (3) is rejected.",
        "- Timestamp for each milestone recorded once and never overwritten",
        "(idempotent replay protection: if server accidentally sends the",
        "same status twice the second call fails on transition check)."
      ],
      "discriminator": [
        109,
        168,
        36,
        134,
        144,
        246,
        63,
        127
      ],
      "accounts": [
        {
          "name": "serverAuthority",
          "writable": true,
          "signer": true
        },
        {
          "name": "deliveryTrace",
          "docs": [
            "DeliveryTrace must already exist."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  101,
                  108,
                  105,
                  118,
                  101,
                  114,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "delivery_trace.order_id",
                "account": "deliveryTrace"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "args",
          "type": {
            "defined": {
              "name": "updateStatusArgs"
            }
          }
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "deliveryTrace",
      "discriminator": [
        176,
        225,
        70,
        46,
        156,
        59,
        50,
        100
      ]
    },
    {
      "name": "productTrace",
      "discriminator": [
        193,
        190,
        92,
        94,
        91,
        206,
        126,
        98
      ]
    }
  ],
  "events": [
    {
      "name": "deliveryInitialized",
      "discriminator": [
        26,
        162,
        214,
        34,
        184,
        90,
        203,
        252
      ]
    },
    {
      "name": "deliveryStatusUpdated",
      "discriminator": [
        100,
        220,
        216,
        121,
        233,
        224,
        182,
        51
      ]
    },
    {
      "name": "productNftMinted",
      "discriminator": [
        142,
        202,
        51,
        44,
        207,
        186,
        44,
        128
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "emptyOrderId",
      "msg": "Order ID cannot be empty"
    },
    {
      "code": 6001,
      "name": "emptyNfcUid",
      "msg": "NFC UID cannot be empty"
    },
    {
      "code": 6002,
      "name": "emptyProductName",
      "msg": "Product name cannot be empty"
    },
    {
      "code": 6003,
      "name": "emptyMetadataUri",
      "msg": "Metadata URI cannot be empty"
    },
    {
      "code": 6004,
      "name": "emptyFarmerId",
      "msg": "Farmer ID cannot be empty"
    },
    {
      "code": 6005,
      "name": "emptyDeliveryPartnerId",
      "msg": "Delivery partner ID cannot be empty"
    },
    {
      "code": 6006,
      "name": "emptyConsumerId",
      "msg": "Consumer ID cannot be empty"
    },
    {
      "code": 6007,
      "name": "stringTooLong",
      "msg": "A string argument exceeds the maximum allowed length"
    },
    {
      "code": 6008,
      "name": "invalidStatusTransition",
      "msg": "Status can only advance by one step at a time (no skipping)"
    },
    {
      "code": 6009,
      "name": "invalidStatusValue",
      "msg": "new_status value is out of range (max 3)"
    },
    {
      "code": 6010,
      "name": "alreadyInitialized",
      "msg": "Delivery already initialized for this order"
    },
    {
      "code": 6011,
      "name": "orderIdMismatch",
      "msg": "order_id in args does not match the loaded ProductTrace PDA"
    },
    {
      "code": 6012,
      "name": "notInitialized",
      "msg": "DeliveryTrace has not been initialized yet"
    }
  ],
  "types": [
    {
      "name": "deliveryInitialized",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "orderId",
            "type": "string"
          },
          {
            "name": "nfcUid",
            "type": "string"
          },
          {
            "name": "timestamp",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "deliveryStatusUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "orderId",
            "type": "string"
          },
          {
            "name": "newStatus",
            "type": "u8"
          },
          {
            "name": "timestamp",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "deliveryTrace",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "orderId",
            "type": "string"
          },
          {
            "name": "nftMint",
            "type": "pubkey"
          },
          {
            "name": "nfcUid",
            "type": "string"
          },
          {
            "name": "status",
            "type": "u8"
          },
          {
            "name": "farmerId",
            "type": "string"
          },
          {
            "name": "deliveryPartnerId",
            "type": "string"
          },
          {
            "name": "consumerId",
            "type": "string"
          },
          {
            "name": "initializedAt",
            "type": "i64"
          },
          {
            "name": "pickedUpAt",
            "type": "i64"
          },
          {
            "name": "inTransitAt",
            "type": "i64"
          },
          {
            "name": "deliveredAt",
            "type": "i64"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "initDeliveryArgs",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "orderId",
            "type": "string"
          },
          {
            "name": "nfcUid",
            "type": "string"
          },
          {
            "name": "farmerId",
            "type": "string"
          },
          {
            "name": "deliveryPartnerId",
            "type": "string"
          },
          {
            "name": "consumerId",
            "type": "string"
          }
        ]
      }
    },
    {
      "name": "mintProductArgs",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "orderId",
            "type": "string"
          },
          {
            "name": "productName",
            "type": "string"
          },
          {
            "name": "metadataUri",
            "type": "string"
          },
          {
            "name": "farmerWallet",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "productNftMinted",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "orderId",
            "type": "string"
          },
          {
            "name": "nftMint",
            "type": "pubkey"
          },
          {
            "name": "timestamp",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "productTrace",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "orderId",
            "type": "string"
          },
          {
            "name": "nftMint",
            "type": "pubkey"
          },
          {
            "name": "farmerWallet",
            "type": "pubkey"
          },
          {
            "name": "productName",
            "type": "string"
          },
          {
            "name": "metadataUri",
            "type": "string"
          },
          {
            "name": "createdAt",
            "type": "i64"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "updateStatusArgs",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "newStatus",
            "type": "u8"
          }
        ]
      }
    }
  ]
};
