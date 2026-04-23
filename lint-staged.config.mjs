export default {
  '*.{cjs,mjs,js,ts,vue}': () =>
    'pnpm --filter @tileserver-rs/client run lint',
  '*.rs': () => [
    'cargo fmt --all -- --check',
    'cargo clippy --all-targets --all-features -- -D warnings',
  ],
};
