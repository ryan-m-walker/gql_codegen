export type Role = 'ADMIN' | 'USER' | '%future added value';

export type Species = 'DOG' | 'CAT' | '%future added value';

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

/**
 *  They is just peoples 
 */
export type Person = {
  __typename?: 'Person';
  id: string;
  name: string;
  role: Role | null;
  friends: Array<Person>;
  /**
   *  They is just pets 
   */
  pets: Array<Animal | null> | null;
  emails: Array<string> | null;
  age: number | null;
};

export type Query = {
  __typename?: 'Query';
  person: Person | null;
  persons: Array<Person>;
  hello: string;
  hellos: Array<string>;
  pets: Array<Animal>;
};

export type TestInput = {
  __typename?: 'TestInput';
  id: string;
  name: string;
  role: Role | null;
};

export type Animal = {
  id: string;
  species: Species;
  name: string;
};

export type Pet = Cat | Dog | Person;

export type Date = unknown;

export type TestQuery = {
  pets: Array<{
    __typename?: 'Animal';
    id: string;
    breed: string;
    name: string;
  }>;
};

import { z } from 'zod';

export const RoleSchema = z.enum(['ADMIN', 'USER']);

export const SpeciesSchema = z.enum(['DOG', 'CAT']);

export const AnimalSchema: z.ZodType<Animal> = z.object({
  id: z.string(),
  species: SpeciesSchema,
  name: z.string(),
});

export const PetSchema = z.union([CatSchema, DogSchema, PersonSchema]);

export const DogSchema: z.ZodType<Dog> = z.object({
  __typename: z.literal('Dog').nullish(),
  id: z.string(),
  species: SpeciesSchema,
  name: z.string(),
  breed: z.string(),
}).merge(AnimalSchema);

export const CatSchema: z.ZodType<Cat> = z.object({
  __typename: z.literal('Cat').nullish(),
  id: z.string(),
  species: SpeciesSchema,
  name: z.string(),
  color: z.string(),
}).merge(AnimalSchema);

/**
 *  They is just peoples 
 */
export const PersonSchema: z.ZodType<Person> = z.object({
  __typename: z.literal('Person').nullish(),
  id: z.string(),
  name: z.string(),
  role: RoleSchema.nullish(),
  friends: PersonSchema.array(),
  /**
   *  They is just pets 
   */
  pets: AnimalSchema.nullish().array().nullish(),
  emails: z.string().array().nullish(),
  age: z.number().int().nullish(),
});

export const QuerySchema: z.ZodType<Query> = z.object({
  __typename: z.literal('Query').nullish(),
  person: PersonSchema.nullish(),
  persons: PersonSchema.array(),
  hello: z.string(),
  hellos: z.string().array(),
  pets: AnimalSchema.array(),
});

export const TestInputInputSchema: z.ZodType<TestInput> = z.object({
  __typename: z.literal('TestInput').nullish(),
  id: z.string(),
  name: z.string(),
  role: RoleSchema.nullish(),
});

export const DateSchema = z.unknown();

