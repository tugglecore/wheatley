const { withKumaUI } = require("@kuma-ui/next-plugin");

// const nextConfig = {
//   reactStrictMode: true,
// };

const nextConfig = {
	output: 'export',
	images: {
		unoptimized: true
	}
}
	
module.exports = withKumaUI(nextConfig, {})
