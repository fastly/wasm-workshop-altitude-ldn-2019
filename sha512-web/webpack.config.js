const path = require('path');
const { CleanWebpackPlugin } = require('clean-webpack-plugin');
const HtmlWebpackPlugin = require('html-webpack-plugin');

module.exports = {
    mode: 'development',
    entry: './src/index.js',
    devServer: {
        contentBase: path.resolve(__dirname, 'dist'),
        watchContentBase: true
    },
    plugins: [
        new CleanWebpackPlugin(),
        new HtmlWebpackPlugin({
            title: 'SHA512 with AssemblyScript',
            template: './src/index.html',
        })
    ],
    module: {
        rules: [
            {
                test: /\.wasm$/,
                type: 'javascript/auto',
                use: {
                    loader: 'file-loader',
                    options: {
                        filename: '[name].[hash].wasm'
                    }
                }
            }
        ]
    },
    output: {
        filename: 'main.[hash].js',
        path: path.resolve(__dirname, 'dist')
    }
};
