import { fileURLToPath } from "node:url";
import createJiti from "jiti";
const jiti = createJiti(fileURLToPath(import.meta.url));


// Import env here to validate during build. Using jiti@^1 we can import .ts files :)
jiti("./src/env");

/** @type {import('next').NextConfig} */
export default {
  /* config options here */
  output: "export",
  transpilePackages: ["@t3-oss/env-nextjs", "@t3-oss/env-core"],
  images: {
    unoptimized: true,
  },
  assetPrefix: process.env.NEXT_PUBLIC_ENV === "production" ? undefined : "http://localhost:3000",

};

