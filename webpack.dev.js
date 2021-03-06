const path = require('path');
const CopyPlugin = require('copy-webpack-plugin');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

module.exports = {
    entry: './src/index.ts',
    mode: 'development',
    module: {
        rules: [
            {
                test: /\.ts$/,
                use: 'ts-loader',
                exclude: /node_modules/,
            },
            {
                test: /\.wasm$/,
                type: 'webassembly/sync',
            },
        ],
    },
    experiments: {
        syncWebAssembly: true,
    },
    plugins: [
        new CopyPlugin({
            patterns: [
                { from: 'static', to: path.join(__dirname, 'dist') },
            ],
        }),
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, './backend'),
            outDir: '../dist',
            outName: 'tower_defense',
            withTypescript: true,
        }),
    ],
    devtool: 'source-map',
    devServer: {
        contentBase: path.join(__dirname, 'dist'),
        compress: true,
        port: 9000,
    },
    resolve: {
        extensions: ['.ts', '.js'],
    },
    output: {
        filename: 'bundle.js',
        path: path.resolve(__dirname, 'dist'),
    },
};
