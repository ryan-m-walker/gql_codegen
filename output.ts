export type Role = 'ADMIN' | 'USER' | '%future added value';

export type Species = 'DOG' | 'CAT' | '%future added value';

export type Dog = Animal & {
  __typename: 'Dog';
  id: string;
  species: Species;
  name: string;
  breed: string;
};

export type Cat = Animal & {
  __typename: 'Cat';
  id: string;
  species: Species;
  name: string;
  color: string;
};

export type Person = {
  __typename: 'Person';
  id: string;
  name: string;
  role: Role | null;
  friends: Array<Person>;
  pets: Array<Animal | null> | null;
  emails: Array<string> | null;
  age: number | null;
};

export type Query = {
  __typename: 'Query';
  person: Person | null;
  persons: Array<Person>;
  hello: string;
  hellos: Array<string>;
};

export type Animal = {
  id: string;
  species: Species;
  name: string;
};

export type Pet = Cat | Dog | Person;

export type TestQuery = {
  persons: Array<{
    pets: Array<{
      __typename?: 'Dog';
      breed: string;
      color: string;
    } | null> | null;
  }>;
};

