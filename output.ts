export type Role = 'ADMIN' | 'USER' | '%future added value';

export type Species = 'DOG' | 'CAT' | '%future added value';

export type Animal = {
  id: string;
  species: Species;
  name: string;
};

export type Dog = Animal & {
  __typename?: 'Dog';
  id: string;
  species: Species;
  name: string;
  breed: string;
};

export type Cat = Animal & {
  __typename?: 'Cat';
  id: string;
  species: Species;
  name: string;
  color: string;
};

export type Pet = Cat | Dog;

export type Person = {
  __typename?: 'Person';
  id: string;
  name: string;
  role: Role | null;
  friends: Array<Person>;
  pets: Array<Animal> | null;
};

export type Query = {
  __typename?: 'Query';
  person: Person | null;
  persons: Array<Person>;
  hello: string;
  hellos: Array<string>;
};

export type TestQuery = {
  __typename?: 'Query';
  hello: string;
  yo: string;
  persons: {
    __typename?: 'Person';
    id: string;
    friends: {
      __typename?: 'Person';
      id: string;
      name: string;
      role: Role | null;

    };
    pets: Array<{
      __typename?: 'Animal';
      species: Species;

    }> | null;
    name: string;
    role: Role | null;

  };
};

export type TestQuery = {
  __typename?: 'Query';
  hello: string;
  yo: string;
  persons: {
    __typename?: 'Person';
    id: string;
    friends: {
      __typename?: 'Person';
      id: string;
      name: string;
      role: Role | null;

    };
    pets: Array<{
      __typename?: 'Animal';
      species: Species;

    }> | null;
    name: string;
    role: Role | null;

  };
};

