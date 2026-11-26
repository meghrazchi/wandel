import { useId } from "react";

interface WandelIconProps {
  className?: string;
  /** "badge" renders the full gradient squircle app icon; "mark" renders only the W emblem + spark in currentColor. */
  variant?: "badge" | "mark";
}

export function WandelIcon({
  className = "h-8 w-8",
  variant = "badge",
}: WandelIconProps) {
  const uid = useId();
  const bgId = `wandel-bg-${uid}`;
  const shineId = `wandel-shine-${uid}`;

  if (variant === "mark") {
    return (
      <svg
        xmlns="http://www.w3.org/2000/svg"
        viewBox="80 96 352 300"
        className={className}
        aria-hidden="true"
      >
        <path
          d="M118 164 L168 348 L224 232 L256 296 L288 232 L344 348 L394 164"
          fill="none"
          stroke="currentColor"
          strokeWidth="44"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
        <circle cx="386" cy="126" r="18" fill="currentColor" opacity="0.9" />
      </svg>
    );
  }

  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 512 512"
      className={className}
      aria-hidden="true"
    >
      <defs>
        <linearGradient id={bgId} x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor="#4f46e5" />
          <stop offset="55%" stopColor="#6d28d9" />
          <stop offset="100%" stopColor="#9333ea" />
        </linearGradient>
        <linearGradient id={shineId} x1="0%" y1="0%" x2="0%" y2="100%">
          <stop offset="0%" stopColor="#ffffff" stopOpacity="0.22" />
          <stop offset="100%" stopColor="#ffffff" stopOpacity="0.0" />
        </linearGradient>
      </defs>
      <rect x="24" y="24" width="464" height="464" rx="112" fill={`url(#${bgId})`} />
      <rect x="24" y="24" width="464" height="232" rx="112" fill={`url(#${shineId})`} />
      <path
        d="M118 164 L168 348 L224 232 L256 296 L288 232 L344 348 L394 164"
        fill="none"
        stroke="#ffffff"
        strokeWidth="38"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <circle cx="386" cy="126" r="16" fill="#e0e7ff" />
    </svg>
  );
}
