/** @type {import('webpack').Configuration} */

const path = require("path");
const CopyPlugin = require('copy-webpack-plugin')

module.exports = {
  entry: path.resolve(__dirname, "src", "App.tsx"),
  mode: process.env.NODE_ENV ?? "development",
  target: "web",
  module: {
    rules: [
      {
        test: /\.(ts|tsx)/,
        include: path.resolve(__dirname, "src"),
        exclude: /node_modules/,
        use: ["ts-loader"]
      },
      {
        test: /\.(css)/,
        include: path.resolve(__dirname, "src"),
        exclude: /node_modules/,
        use: ["style-loader", "css-loader", "postcss-loader"]
      },
      {
        test: /\.(png|jpe?g|gif|svg|woff2?|ttf|otf)$/i,
        include: path.resolve(__dirname, "src"),
        type: "asset/resource",
        dependency: { not: ["url"] }
      }
    ]
  },
  plugins: [
    new CopyPlugin({
      patterns: [
        { from: path.resolve(__dirname, "index.html"), to: "index.html" },
        { from: path.resolve(__dirname, "static/"), to: "static/" },
      ],
      options: {
        concurrency: 100
      }
    }),
  ],
  resolve: {
    extensions: [".css", ".js", ".jsx", ".tsx", ".ts", ".cjs"],
    alias: {
      "tailwindcss/resolveConfig": "tailwindcss/resolveConfig.js",
      "~": path.resolve(__dirname)
    }
  },
  output: {
    filename: "bundle.js",
    path: path.resolve(__dirname, "dist"),
    publicPath: "/",
    library: {
      type: "umd"
    }
  },
  devServer: {
    static: path.resolve(__dirname, "dist"),
    port: 3000,
    https: true,
    headers: {
      "Cross-Origin-Opener-Policy": "same-origin",
      "Cross-Origin-Embedder-Policy": "require-corp"
    },
    proxy: {
      "/api": "http://localhost:8080",
      "/ws": "http://localhost:8080"
    },
    historyApiFallback: true
  },
  optimization: {
    usedExports: false
  },
};
