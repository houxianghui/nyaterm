import { renderToString } from "react-dom/server";
import { describe, expect, it } from "vitest";
import NyaTermLogo from "./NyaTermLogo";

describe("NyaTermLogo", () => {
  it("renders the bundled app icon image", () => {
    const html = renderToString(<NyaTermLogo className="h-5 w-5" />);

    expect(html).toContain("<img");
    expect(html).toContain('alt="NyaTerm"');
    expect(html).toContain("app-icon");
    expect(html).toContain("rounded-[22%]");
    expect(html).toContain("h-5 w-5");
    expect(html).toContain('draggable="false"');
  });
});
