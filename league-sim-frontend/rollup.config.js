import { wasm } from '@rollup/plugin-wasm';

export default {
    input: 'src/index.js',
    output: {
        dir: 'output',
        format: 'cjs'
    },
    plugins: [wasm()]
};