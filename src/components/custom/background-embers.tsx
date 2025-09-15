"use client";

import { cn } from "@/lib/utils";
import { motion } from "motion/react";
import { useEffect, useMemo, useState } from "react";
import Particles, { initParticlesEngine } from "@tsparticles/react";
import { loadSlim } from "@tsparticles/slim";

interface BackgroundEmbersProps {
  className?: string;
  contentClassName?: string;
  children?: React.ReactNode;
  particlesEnabled?: boolean;
}

const BackgroundEmbers = ({
  className = "",
  children,
  contentClassName = "",
  particlesEnabled = true,
}: BackgroundEmbersProps) => {
  const [init, setInit] = useState(false);

  // Initialize tsParticles engine
  useEffect(() => {
    initParticlesEngine(async (engine) => {
      await loadSlim(engine);
    }).then(() => {
      setInit(true);
    });
  }, []);

  const particlesLoaded = async (container?: any): Promise<void> => {
    console.log("Fire particles loaded:", container);
  };

  // Fire ember particles configuration
  const options: any = useMemo(
    () => ({
      background: {
        color: {
          value: "#4a0000", // Deep red background
        },
      },
      fpsLimit: 120,
      interactivity: {
        events: {
          onClick: {
            enable: false,
          },
          onHover: {
            enable: false,
          },
        },
      },
      particles: {
        color: {
          value: [
            "#ff4500", // Orange red
            "#ff6347", // Tomato
            "#ff7f50", // Coral
            "#ff8c00", // Dark orange
            "#ff6b35", // Red orange
            "#ff4500", // Orange red
            "#ff0000", // Pure red
            "#ffa500", // Orange
          ],
        },
        move: {
          direction: "top",
          enable: true,
          outModes: {
            default: "out",
          },
          random: true,
          speed: 15, // Much faster moving ember particles
          straight: false,
        },
        number: {
          density: {
            enable: true,
            area: 600,
          },
          value: 200, // More particles for dense ember effect
        },
        opacity: {
          value: 0.9,
          random: true,
        },
        shape: {
          type: "circle",
        },
        size: {
          value: { min: 0.5, max: 3 }, // Smaller particles like real embers
          random: true,
        },
        twinkle: {
          particles: {
            enable: true,
            frequency: 0.1, // More frequent twinkling for embers
            opacity: 1,
          },
        },
        life: {
          count: 0,
          delay: {
            value: 0,
            sync: false,
          },
          duration: {
            value: 0,
            sync: false,
          },
        },
      },
      detectRetina: true,
    }),
    []
  );

  return (
    <div
      className={cn("relative h-full w-full", className)}
      style={{
        background: `
          radial-gradient(circle at center, rgba(0, 0, 0, 0.1) 0%, rgba(0, 0, 0, 0.8) 100%),
          linear-gradient(135deg, #4A1E00 0%, #2d0000 50%, #1A0000 100%)
        `,
      }}
    >
      {/* Vignette effect overlay */}
      <div
        className="absolute inset-0 pointer-events-none z-10"
        style={{
          background: `
            radial-gradient(circle at center, 
              transparent 0%, 
              transparent 40%, 
              rgba(0, 0, 0, 0.3) 70%, 
              rgba(0, 0, 0, 0.8) 100%
            )
          `,
        }}
      />

      {/* tsParticles Fire Effect */}
      {init && particlesEnabled && (
        <Particles
          id="fire-particles"
          particlesLoaded={particlesLoaded}
          options={options}
          className="absolute inset-0 z-0"
          style={{
            position: "absolute",
            top: 0,
            left: 0,
            width: "100%",
            height: "100%",
          }}
        />
      )}
      {/* background color to replicate particle color when particles are disabled */}
      {!particlesEnabled && (
        <div
          className="absolute inset-0 z-0"
          style={{ background: "#4a0000" }}
        ></div>
      )}

      {/* Additional fire glow effect at bottom */}
      <motion.div
        className="absolute bottom-0 left-0 w-full h-40 pointer-events-none z-10"
        style={{
          background: `
            linear-gradient(to top,
              rgba(255, 100, 0, 0.4) 0%,
              rgba(255, 150, 0, 0.3) 30%,
              rgba(255, 200, 0, 0.2) 60%,
              transparent 100%
            )
          `,
          filter: "blur(30px)",
        }}
        animate={{
          opacity: [0.4, 0.7, 0.4],
        }}
        transition={{
          duration: 1.5,
          repeat: Infinity,
          ease: "easeInOut",
        }}
      />

      {/* Additional atmospheric elements */}
      <motion.div
        className="absolute top-0 left-0 w-full h-full pointer-events-none z-10"
        animate={{
          background: [
            "radial-gradient(circle at 30% 70%, rgba(255, 100, 0, 0.1) 0%, transparent 50%)",
            "radial-gradient(circle at 70% 30%, rgba(255, 150, 0, 0.1) 0%, transparent 50%)",
            "radial-gradient(circle at 30% 70%, rgba(255, 100, 0, 0.1) 0%, transparent 50%)",
          ],
        }}
        transition={{
          duration: 6,
          repeat: Infinity,
          ease: "easeInOut",
        }}
      />

      {/* Bottom fire layer */}
      <motion.div
        className="absolute bottom-0 left-0 w-full h-32 bg-gradient-to-t from-orange-900/40 to-transparent z-10"
        animate={{
          opacity: [0.3, 0.6, 0.3],
        }}
        transition={{
          duration: 2,
          repeat: Infinity,
          ease: "easeInOut",
        }}
      />

      {/* Content layer */}
      <div
        className={cn(
          "absolute inset-0 z-20 overflow-y-auto py-12 pb-16",
          contentClassName
        )}
      >
        {children}
      </div>
    </div>
  );
};

export default BackgroundEmbers;
