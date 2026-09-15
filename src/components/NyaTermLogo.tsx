import type React from "react";
import appIconUrl from "@/assets/app-icon.png";
import { cn } from "@/lib/utils";

interface NyaTermLogoProps extends React.ImgHTMLAttributes<HTMLImageElement> {
  className?: string;
}

export default function NyaTermLogo({ className, ...props }: NyaTermLogoProps) {
  return (
    <img
      src={appIconUrl}
      alt="NyaTerm"
      draggable={false}
      className={cn("rounded-[22%]", className)}
      {...props}
    />
  );
}
