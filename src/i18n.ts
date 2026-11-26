import { Locale } from "./types";

const DICT: Record<Locale, Record<string, string>> = {
  en: {
    subtitle: "Fast Parallel Media Converter",
    watch: "Watch",
    history: "History",
    dropTitle: "Drop images, videos, PDFs, or folders here",
    dropActive: "Drop to add to Wandel",
    inspecting: "Inspecting files...",
    addFiles: "Add Files",
    addFolder: "Add Folder",
    commandPlaceholder: "Type a command or search actions...",
    noCommands: "No matching commands.",
  },
  de: {
    subtitle: "Schneller Paralleler Medienkonverter",
    watch: "Überwachen",
    history: "Verlauf",
    dropTitle: "Bilder, Videos, PDFs oder Ordner hier ablegen",
    dropActive: "Loslassen zum Hinzufügen",
    inspecting: "Dateien werden analysiert...",
    addFiles: "Dateien wählen",
    addFolder: "Ordner wählen",
    commandPlaceholder: "Befehl eingeben oder Aktion suchen...",
    noCommands: "Keine passenden Befehle.",
  },
};

export function t(locale: Locale, key: string): string {
  return DICT[locale]?.[key] ?? DICT.en[key] ?? key;
}
