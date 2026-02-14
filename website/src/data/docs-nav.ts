export interface DocsNavItem {
  title: string;
  href: string;
  description: string;
}

export interface DocsNavGroup {
  group: string;
  items: DocsNavItem[];
}

export const docsNav: DocsNavGroup[] = [
  {
    group: 'Getting Started',
    items: [
      {
        title: 'About',
        href: '/docs/about',
        description: 'Philosophy, design principles, and inspirations.',
      },
      {
        title: 'Installation',
        href: '/docs/installation',
        description: 'Install SGC via npm, pnpm, or yarn.',
      },
      {
        title: 'Configuration',
        href: '/docs/configuration',
        description: 'Set up your codegen.config.ts file.',
      },
    ],
  },
  {
    group: 'Generators',
    items: [
      {
        title: 'schema-types',
        href: '/docs/generator-schema-types',
        description: 'Generate TypeScript types from your GraphQL schema.',
      },
      {
        title: 'operation-types',
        href: '/docs/generator-operation-types',
        description: 'Generate typed operations, fragments, and variables.',
      },
      {
        title: 'typed-documents',
        href: '/docs/generator-typed-documents',
        description: 'Generate typed document constants for runtime use.',
      },
    ],
  },
  {
    group: 'Guides',
    items: [
      {
        title: 'Schema Sources',
        href: '/docs/schema',
        description: 'Specify where to find your GraphQL schema files.',
      },
      {
        title: 'Documents',
        href: '/docs/documents',
        description: 'Configure which files contain your GraphQL operations.',
      },
      {
        title: 'CLI Usage',
        href: '/docs/cli',
        description: 'Run SGC from the command line with flags and options.',
      },
    ],
  },
  {
    group: 'Migration',
    items: [
      {
        title: 'From graphql-codegen',
        href: '/docs/migration',
        description: 'Migrate from graphql-codegen to SGC.',
      },
    ],
  },
];

export const docsNavFlat: DocsNavItem[] = docsNav.flatMap((g) => g.items);
