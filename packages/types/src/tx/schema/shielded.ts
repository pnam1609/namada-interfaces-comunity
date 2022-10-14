import BN from "bn.js";
import { ShieldedDataProps, ShieldedProps } from "../types";

export class ShieldedTransferMsgValue {
  txid: Uint8Array;
  data: Uint8Array;

  constructor(properties: ShieldedProps) {
    this.txid = properties.txId;
    this.data = properties.data;
  }
}

export const ShieldedTransferMsgSchema = new Map([
  [
    ShieldedTransferMsgValue,
    {
      kind: "struct",
      fields: [
        ["txid", "[]"],
        ["data", "[]"],
      ],
    },
  ],
]);

export class ShieldedDataMsgValue {
  overwintered: boolean;
  version: string;
  version_group_id: string;
  vin: Uint8Array;
  vout: Uint8Array;
  lock_time: BN;
  expiry_height: BN;
  value_balance: BN;
  shielded_spends: Uint8Array;
  shielded_converts: Uint8Array;
  shielded_outputs: Uint8Array;
  join_splits: string;
  join_split_pubkey?: Uint8Array;
  join_split_sig?: Uint8Array;
  binding_sig?: Uint8Array;

  constructor(properties: ShieldedDataProps) {
    this.overwintered = properties.overwintered;
    this.version = properties.version;
    this.version_group_id = properties.versionGroupId;
    this.vin = properties.vin;
    this.vout = properties.vout;
    this.lock_time = new BN(properties.lockTime, 64);
    this.expiry_height = new BN(properties.expiryHeight, 64);
    this.value_balance = new BN(properties.valueBalance, 64);
    this.shielded_spends = properties.shieldedSpends;
    this.shielded_converts = properties.shieldedConverts;
    this.shielded_outputs = properties.shieldedOutputs;
    this.join_splits = properties.joinSplits;
    this.join_split_pubkey = properties.joinSplitPubKey;
    this.join_split_sig = properties.joinSplitSig;
    this.binding_sig = properties.bindingSig;
  }
}

export const ShieldedDataMsg = new Map([
  [
    ShieldedDataMsgValue,
    [
      ["overwintered", "boolean"],
      ["version", "string"],
      ["version_group_id", []],
      ["vin", []],
      ["vout", []],
      ["lock_time", "u64"],
      ["expiry_height", "u64"],
      ["value_balance", "u64"],
      ["shielded_spends", []],
      ["shielded_converts", []],
      ["shielded_outputs", []],
      ["join_splits", "string"],
      ["join_split_pubkey", []],
      ["join_split_sig", []],
      ["binding_sig", []],
    ],
  ],
]);
