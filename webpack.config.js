const path = require('path');
const CopyWebpackPlugin = require('copy-webpack-plugin');

module.exports = {
  mode: 'production',
  entry: './js/walletconnect-entry.js',
  output: {
    path: path.resolve(__dirname, 'js'),
    filename: 'walletconnect-bundle.js',
    library: {
      name: 'WalletConnectLib',
      type: 'umd',
    },
    globalObject: 'this',
    chunkFormat: false,
  },
  optimization: {
    splitChunks: false,
  },
  experiments: {
    outputModule: false,
  },
  resolve: {
    conditionNames: ['require', 'node', 'default'],
    fallback: {
      "buffer": require.resolve("buffer"),
      "stream": require.resolve("stream-browserify"),
      "crypto": require.resolve("crypto-browserify"),
      "util": require.resolve("util"),
      "process": require.resolve("process/browser"),
    }
  },
  plugins: [
    new (require('webpack').ProvidePlugin)({
      Buffer: ['buffer', 'Buffer'],
      process: 'process/browser',
    }),
    new (require('webpack').DefinePlugin)({
      'typeof exports': JSON.stringify('object'),
    }),
    new CopyWebpackPlugin({
      patterns: [
        {
          from: path.resolve(__dirname, 'node_modules/@walletconnect/ethereum-provider/dist/index.umd.js'),
          to: path.resolve(__dirname, 'js/walletconnect-umd.js'),
        },
      ],
    }),
  ],
};

