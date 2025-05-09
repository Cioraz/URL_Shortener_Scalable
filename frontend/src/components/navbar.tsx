"use client";

import React from "react";
import { FloatingDock } from "@/components/ui/floating-dock";
import { IconBolt, IconSearch, IconSettings } from "@tabler/icons-react";

export const navItems = [
  { title: "Generate", icon: <IconBolt />, href: "/generate" },
  { title: "Retrieve", icon: <IconSearch />, href: "/retrieve" },
  { title: "Custom", icon: <IconSettings />, href: "/custom" },
];

export function Navbar() {
  return (
    <div className="absolute top-20 left-0 right-0 flex justify-center z-50">
      <FloatingDock
        items={navItems}
        desktopClassName="bg-gray-200 dark:bg-gray-900 backdrop-blur-sm"
      />
    </div>
  );
}
