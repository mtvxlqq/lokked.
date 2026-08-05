import { PingCard } from "@/components/PingCard";

/**
 * The app's only screen for now. Later this becomes the dashboard that the
 * timer, flashcard and statistics routes hang off.
 */
export function Home() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-center gap-8 p-8">
      <h1 className="text-4xl font-semibold tracking-tight text-content-100">
        StudyApp
      </h1>
      <PingCard />
    </main>
  );
}
