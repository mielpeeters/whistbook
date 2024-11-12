const path = require('path');

module.exports = {
  entry: {
    main: './js/index.js',
    qr: './js/qrscanner.js',
  },
  output: {
    filename: '[name].bundle.js',
    path: path.resolve(__dirname, 'public/src'),
  },
  mode: 'production',
};
