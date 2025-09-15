"use client";

import { cn } from "@/lib/utils";
import { motion } from "motion/react";
import BackgroundEmbers from "./background-embers";

const HeroFireBanner = ({
  title = "NEREVAR",
  subtitle = "Rise from the ashes",
  className = "",
  children,
}: {
  title?: string;
  subtitle?: string;
  className?: string;
  children?: React.ReactNode;
}) => {
  return (
    <BackgroundEmbers className={cn("min-h-full", className)}>
      {/* Main content */}
      <div className="flex items-center justify-center min-h-full px-4 py-8">
        <motion.div
          className="text-center max-w-4xl mx-auto"
          initial={{ opacity: 0, y: 50 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 1, delay: 0.5 }}
        >
          {/* Title with fire effect */}
          <motion.h1
            className="text-6xl md:text-8xl font-sovngarde mb-6 text-white bg-clip-text font-bold"
            style={{
              textShadow:
                "0 0 20px rgba(255, 100, 0, 0.5), 2px 2px 4px rgba(0,0,0,0.8)",
            }}
            animate={{
              backgroundPosition: ["0% 50%", "100% 50%", "0% 50%"],
            }}
            transition={{
              duration: 2,
              repeat: Infinity,
              ease: "easeInOut",
            }}
          >
            {title}
          </motion.h1>

          {/* Subtitle with weathered look */}
          <motion.p
            className="text-xl md:text-2xl text-white mb-8 font-sovngarde"
            style={{
              textShadow:
                "0 0 10px rgba(255, 100, 0, 0.3), 1px 1px 2px rgba(0,0,0,0.8)",
            }}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ duration: 1, delay: 1 }}
          >
            {subtitle}
          </motion.p>
          {children}
        </motion.div>
      </div>
    </BackgroundEmbers>
  );
};

export default HeroFireBanner;
