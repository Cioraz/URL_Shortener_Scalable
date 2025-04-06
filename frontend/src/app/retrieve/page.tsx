"use client";
import { Input } from "@/components/ui/input";

export default function GeneratePage() {
  return (
    <div className="min-h-screen flex flex-col">
      <div className="flex-1 flex items-center justify-center">
        <div className="text-center">
          <h1 className="text-2xl font-bold mb-4">
            Retrieve corresponding long URL
          </h1>
          <Input className="bg-white dark:bg-slate-900" />
        </div>
      </div>
    </div>
  );
}
