import { z } from 'zod';

export const RoleSchema = z.enum(['ADMIN', 'USER']);

export const SpeciesSchema = z.enum(['DOG', 'CAT']);

export const AnimalSchema = z.object({
  id: z.string(),
  species: SpeciesSchema,
  name: z.string(),
});

export const DogSchema = z.object({
  id: z.string(),
  species: SpeciesSchema,
  name: z.string(),
  breed: z.string(),
}).merge(AnimalSchema);

export const CatSchema = z.object({
  id: z.string(),
  species: SpeciesSchema,
  name: z.string(),
  color: z.string(),
}).merge(AnimalSchema);

/**
 *  They is just peoples 
 */
export const PersonSchema = z.object({
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

export const QuerySchema = z.object({
  person: PersonSchema.nullish(),
  persons: PersonSchema.array(),
  hello: z.string(),
  hellos: z.string().array(),
  pets: AnimalSchema.array(),
});

export const TestInputInputSchema = z.object({
  id: z.string(),
  name: z.string(),
  role: RoleSchema.nullish(),
});

export const PetSchema = z.union([CatSchema, DogSchema, PersonSchema]);

export const DateSchema = z.unknown();

