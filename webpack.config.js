const path = require('path');

module.exports = [
  // Farcaster SDK bundle
  {
    mode: 'production',
    entry: './js/farcaster-entry.js',
    output: {
      path: path.resolve(__dirname, 'js'),
      filename: 'farcaster-bundle.js',
      library: {
        name: 'FarcasterSDK',
        type: 'umd',
      },
      globalObject: 'window',
      chunkFormat: false,
    },
    optimization: {
      splitChunks: false,
    },
    resolve: {
      extensions: ['.js', '.json'],
    },
    module: {
      rules: [
        {
          test: /\.js$/,
          exclude: /node_modules\/(?!@farcaster)/,
          use: {
            loader: 'babel-loader',
            options: {
              presets: [
                ['@babel/preset-env', {
                  modules: 'umd',
                }],
              ],
            },
          },
        },
      ],
    },
  },
];

