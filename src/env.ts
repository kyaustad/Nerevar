import { createEnv } from "@t3-oss/env-nextjs";
import { z } from "zod";

export const env = createEnv({
  server: {},
  client: {
    NEXT_PUBLIC_ENV: z.string(),
    //NEXT_PUBLIC_LATEST_TES3MP_WINDOWS_RELEASE: z.string(),
    //NEXT_PUBLIC_NEREVAR_APP_VERSION: z.string(),
    NEXT_PUBLIC_NEREVAR_API_URL: z.string(),
  },
  // If you're using Next.js < 13.4.4, you'll need to specify the runtimeEnv manually
  //   runtimeEnv: {
  //     NEXT_PUBLIC_ENV: process.env.NEXT_PUBLIC_ENV,
  //   },
  //   For Next.js >= 13.4.4, you only need to destructure client variables:
  experimental__runtimeEnv: {
    NEXT_PUBLIC_ENV: process.env.NEXT_PUBLIC_ENV,
    //NEXT_PUBLIC_LATEST_TES3MP_WINDOWS_RELEASE: process.env.NEXT_PUBLIC_LATEST_TES3MP_WINDOWS_RELEASE,
    //NEXT_PUBLIC_NEREVAR_APP_VERSION:
    //  process.env.NEXT_PUBLIC_NEREVAR_APP_VERSION,
    NEXT_PUBLIC_NEREVAR_API_URL: process.env.NEXT_PUBLIC_NEREVAR_API_URL,
  },
});
