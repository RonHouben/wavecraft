import js from '@eslint/js';
import typescript from '@typescript-eslint/eslint-plugin';
import typescriptParser from '@typescript-eslint/parser';
import react from 'eslint-plugin-react';
import reactHooks from 'eslint-plugin-react-hooks';
import reactRefresh from 'eslint-plugin-react-refresh';
import eslintConfigPrettier from 'eslint-config-prettier';
import globals from 'globals';

export default [
  // Base JavaScript rules
  js.configs.recommended,
  
  // TypeScript files
  {
    files: ['**/*.{ts,tsx}'],
    languageOptions: {
      parser: typescriptParser,
      parserOptions: {
        ecmaVersion: 'latest',
        sourceType: 'module',
        ecmaFeatures: { jsx: true },
        project: './tsconfig.json',
      },
      globals: {
        ...globals.browser,
        __APP_VERSION__: 'readonly',
      },
    },
    plugins: {
      '@typescript-eslint': typescript,
      'react': react,
      'react-hooks': reactHooks,
      'react-refresh': reactRefresh,
    },
    rules: {
      // TypeScript strict rules
      ...typescript.configs['recommended'].rules,
      ...typescript.configs['strict'].rules,
      'no-undef': 'off',
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/explicit-function-return-type': 'off',
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      
      // React rules
      ...react.configs.recommended.rules,
      'react/react-in-jsx-scope': 'off', // Not needed with React 17+
      'react/prop-types': 'off', // Using TypeScript
      
      // React Hooks rules (exhaustive deps)
      ...reactHooks.configs.recommended.rules,
      'react-hooks/rules-of-hooks': 'error',
      'react-hooks/exhaustive-deps': 'error',
      'react-hooks/set-state-in-effect': 'off',
      
      // React Refresh (Vite HMR)
      'react-refresh/only-export-components': ['warn', { allowConstantExport: true }],
    },
    settings: {
      react: { version: 'detect' },
    },
  },
  
  // Ignore patterns
  {
    ignores: ['dist/**', 'node_modules/**', '*.config.js'],
  },
  
  // Prettier config must be last to disable conflicting ESLint rules
  eslintConfigPrettier,
];
