import type { MessageKey } from "./i18n/en";

export interface StarterSourcePreset {
  id: string;
  title: string;
  url: string;
  descriptionKey: MessageKey;
  audienceKey: MessageKey;
  badgeKey: MessageKey;
  featured?: boolean;
}

export const STARTER_SOURCES: StarterSourcePreset[] = [
  {
    id: "anthropics-skills",
    title: "anthropics/skills",
    url: "https://github.com/anthropics/skills",
    descriptionKey: "source.presetAnthropicDescription",
    audienceKey: "source.presetAnthropicAudience",
    badgeKey: "source.badgeOfficial",
    featured: true,
  },
  {
    id: "besoeasy-open-skills",
    title: "besoeasy/open-skills",
    url: "https://github.com/besoeasy/open-skills",
    descriptionKey: "source.presetOpenSkillsDescription",
    audienceKey: "source.presetOpenSkillsAudience",
    badgeKey: "source.badgeCommunity",
  },
  {
    id: "am-will-codex-skills",
    title: "am-will/codex-skills",
    url: "https://github.com/am-will/codex-skills",
    descriptionKey: "source.presetCodexDescription",
    audienceKey: "source.presetCodexAudience",
    badgeKey: "source.badgeCodex",
  },
  {
    id: "alirezarezvani-claude-skills",
    title: "alirezarezvani/claude-skills",
    url: "https://github.com/alirezarezvani/claude-skills",
    descriptionKey: "source.presetClaudeDescription",
    audienceKey: "source.presetClaudeAudience",
    badgeKey: "source.badgeClaude",
  },
];

export const DEFAULT_STARTER_SOURCE = STARTER_SOURCES.find((source) => source.featured) ?? STARTER_SOURCES[0];

function normalizeGitUrl(url: string) {
  return url.trim().replace(/\/+$/, "").toLowerCase();
}

export function matchStarterSource(url: string) {
  const normalized = normalizeGitUrl(url);
  return STARTER_SOURCES.find((source) => normalizeGitUrl(source.url) === normalized) ?? null;
}
