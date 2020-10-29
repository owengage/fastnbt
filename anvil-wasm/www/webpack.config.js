const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = {
    entry: {
        main: "./src/index.ts",
        worker: "./src/worker.js",
    },
    output: {
        path: path.resolve(__dirname, "dist"),
        filename: "[name].bundle.js",
    },
    mode: process.env.NODE_ENV || "development",
    plugins: [
        new CopyWebpackPlugin({ patterns: ['src/index.html'] })
    ],
    experiments: {
        asyncWebAssembly: true,
    },
    resolve: {
        extensions: ['.wasm', '.ts', '.js', '.mjs']
    },
    module: {
        rules: [
            {
                test: /\.tsx?$/,
                use: 'ts-loader',
                exclude: /node_modules/,
            },
            {
                test: /\.css$/,
                use: [
                    'style-loader',
                    'css-loader'
                ]
            },
            {
                test: /\.png$/,
                use: [
                    'file-loader'
                ]
            }
        ]
    },
};