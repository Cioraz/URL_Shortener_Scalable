"use client";
import { useState, FormEvent } from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
const GENERATE_ROUTE =
  process.env.NEXT_PUBLIC_GENERATE_ROUTE || "/api/generate";
const API_KEY = process.env.NEXT_PUBLIC_API_KEY || "";

export default function GeneratePage() {
  const [longUrl, setLongUrl] = useState("");
  const [shortUrl, setShortUrl] = useState("");
  const [error, setError] = useState("");

  const handleSubmit = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setError("");
    setShortUrl("");

    if (!GENERATE_ROUTE) {
      setError("API endpoint not configured");
      return;
    }

    try {
      const response = await fetch(GENERATE_ROUTE, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "API-Key": API_KEY,
        },
        body: JSON.stringify({ long_url: longUrl }),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();
      console.log(data);

      if (data.short_url) {
        setShortUrl(data.short_url); // 👈 Use full URL as returned
      } else {
        setError("Failed to generate short URL");
      }
    } catch (error) {
      console.error("Error:", error);
      setError("An error occurred while generating the short URL");
    }
  };

  return (
    <div className="min-h-screen flex flex-col">
      <div className="flex-1 flex items-center justify-center">
        <div className="text-center">
          <h1 className="text-2xl font-bold mb-4">Generate New Short URL</h1>
          <form
            onSubmit={handleSubmit}
            className="flex flex-col items-center space-y-4"
          >
            <Input
              className="bg-white dark:bg-slate-900"
              value={longUrl}
              onChange={(e) => setLongUrl(e.target.value)}
              placeholder="Enter long URL"
            />
            <Button type="submit">Generate</Button>
          </form>
          {shortUrl && (
            <div className="mt-4">
              <p>Short URL:</p>
              <a
                href={shortUrl}
                target="_blank"
                rel="noopener noreferrer"
                className="text-blue-500 hover:underline"
              >
                {shortUrl}
              </a>
            </div>
          )}
          {error && <p className="mt-4 text-red-500">{error}</p>}
        </div>
      </div>
    </div>
  );
}
