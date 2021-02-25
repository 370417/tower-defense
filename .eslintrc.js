module.exports = {
    root: true,
    parser: '@typescript-eslint/parser',
    parserOptions: {
        tsconfigRootDir: __dirname,
        project: ['./tsconfig.json'],
    },
    rules: {
        'comma-dangle': ['error', 'always-multiline'],
        eqeqeq: 2,
        quotes: ['error', 'single', {
            allowTemplateLiterals: true,
        }],
        '@typescript-eslint/unbound-method': 0,
        semi: 2,
    },
    plugins: [
        '@typescript-eslint',
    ],
    extends: [
        'eslint:recommended',
        'plugin:@typescript-eslint/recommended',
        'plugin:@typescript-eslint/recommended-requiring-type-checking',
    ],
    env: {
        browser: true,
        es6: true,
    }
};
