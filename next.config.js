/** @type {import('next').NextConfig} */
const nextConfig = {
  webpack: (config) => {
    config.externals = [
      ...(config.externals || []),
      'bigint',
      'node-gyp-build',
    ];
    return config;
  },
  images: {
    unoptimized: true,
    domains: ['raw.githubusercontent.com'],
  },
};

module.exports = nextConfig;
