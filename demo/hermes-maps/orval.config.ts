import { defineConfig } from 'orval'

export default defineConfig({
  hermes: {
    output: {
      clean: true,
      namingConvention: 'kebab-case',
      mode: 'split',
      target: 'src/api/generated/hermes.ts',
      schemas: 'src/api/generated/schemas',
      client: 'react-query',
      allParamsOptional: true,
      override: {
        mutator: {
          path: 'src/api/fetch.ts',
          name: 'fetchApi',
        },
      },
    },
    input: {
      target: '../../schemas/openapi.json',
    },
  },
})
