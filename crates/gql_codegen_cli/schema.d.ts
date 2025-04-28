export type ID = string;
export type String = string;
export type Int = number;
export type Float = BigInt;
export type Boolean = boolean;
export type Date = Date;

export type Status =  "ACTIVE" | "INACTIVE" | "%future added value";

export interface Node {
  readonly named: String;
  readonly nullable_named: String | null | undefined;
  readonly list: Array<String>;
  readonly list_2: Array<String | null | undefined> | null | undefined;
  readonly list_3: Array<String> | null | undefined;
  readonly list_4: Array<String | null | undefined>;
  readonly id: ID;
  readonly name: String;
}
