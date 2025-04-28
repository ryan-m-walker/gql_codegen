export type ID = string;
export type String = string;
export type Int = number;
export type Float = number;
export type Boolean = boolean;
export type Date = unknown;

export enum Status {
  ACTIVE = "ACTIVE",
  INACTIVE = "INACTIVE",
}

export interface Node {
  named: String;
  nullable_named: String | null | undefined;
  list: Array<String>;
  list_2: Array<String | null | undefined> | null | undefined;
  list_3: Array<String> | null | undefined;
  list_4: Array<String | null | undefined>;
  id: ID;
  name: String;
}
