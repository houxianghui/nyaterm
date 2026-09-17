import { fireEvent, render, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Tab } from "@/types/global";
import TabContextMenu from "./TabContextMenu";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/context/AppContext", () => ({
  useApp: () => ({ updateTab: vi.fn() }),
}));

vi.mock("@/lib/aiEvents", () => ({ openAIAssistant: vi.fn() }));

function makeTab(): Tab {
  return {
    id: "tab-1",
    root: {
      kind: "leaf",
      id: "pane-1",
      paneKind: "terminal",
      type: "Local",
      name: "Local",
      sessionId: "session-1",
    },
    activePaneId: "pane-1",
  } as unknown as Tab;
}

function renderMenu(withTooltip: boolean) {
  const tab = makeTab();
  return render(
    <TabContextMenu
      tab={tab}
      tabs={[tab]}
      tooltipContent={withTooltip ? <div>tip</div> : undefined}
      onDuplicateSession={vi.fn()}
      onMultiplexSshSession={vi.fn()}
      onDuplicateSessionWithCommand={vi.fn()}
      onMultiplexSshSessionWithCommand={vi.fn()}
      onReconnectSession={vi.fn()}
      onDisconnectSession={vi.fn()}
      onSplitSession={vi.fn()}
      onCloseSession={vi.fn()}
      onCloseAll={vi.fn()}
      onCloseInactive={vi.fn()}
      onCloseRight={vi.fn()}
      onSessionInfo={vi.fn()}
      onActivateTab={vi.fn()}
      canCopyIp={false}
      onRenameTab={vi.fn()}
      onCopyTabName={vi.fn()}
      onCopyServerIp={vi.fn()}
    >
      <div data-testid="tab-child">tab</div>
    </TabContextMenu>,
  );
}

async function hoverAndExpectSubmenu(triggerText: string) {
  const trigger = await waitFor(() => {
    const el = document.querySelector(`[data-slot="context-menu-sub-trigger"]`);
    expect(el).not.toBeNull();
    return el as HTMLElement;
  });
  expect(trigger.textContent).toContain(triggerText);
  fireEvent.pointerMove(trigger, { pointerType: "mouse" });
  fireEvent.pointerEnter(trigger, { pointerType: "mouse" });
  fireEvent.mouseOver(trigger);
  await waitFor(
    () => {
      expect(document.querySelector('[data-slot="context-menu-sub-content"]')).not.toBeNull();
    },
    { timeout: 2000 },
  );
  // The sub-content must not be a DOM descendant of the parent content —
  // backdrop-filter on the parent would become the containing block for the
  // sub-content's fixed-position popper and clip it.
  const parent = document.querySelector('[data-slot="context-menu-content"]');
  const sub = document.querySelector('[data-slot="context-menu-sub-content"]');
  expect(parent?.contains(sub)).toBe(false);
}

describe.each([
  ["with tooltip", true],
  ["without tooltip", false],
] as const)("TabContextMenu submenu %s", (_label, withTooltip) => {
  it("opens the set-color submenu on hover", async () => {
    const view = renderMenu(withTooltip);
    fireEvent.contextMenu(view.getByTestId("tab-child"));
    await waitFor(() =>
      expect(document.querySelector('[data-slot="context-menu-content"]')).not.toBeNull(),
    );
    await hoverAndExpectSubmenu("tabCtx.setColor");
  });

  it("opens the AI submenu on hover", async () => {
    const view = renderMenu(withTooltip);
    fireEvent.contextMenu(view.getByTestId("tab-child"));
    await waitFor(() =>
      expect(document.querySelector('[data-slot="context-menu-content"]')).not.toBeNull(),
    );
    const triggers = document.querySelectorAll('[data-slot="context-menu-sub-trigger"]');
    const aiTrigger = Array.from(triggers).find((el) =>
      el.textContent?.includes("ai.title"),
    ) as HTMLElement;
    expect(aiTrigger).toBeTruthy();
    fireEvent.pointerMove(aiTrigger, { pointerType: "mouse" });
    fireEvent.pointerEnter(aiTrigger, { pointerType: "mouse" });
    await waitFor(
      () => {
        const sub = document.querySelector('[data-slot="context-menu-sub-content"]');
        expect(sub?.textContent).toContain("ai.explainRecent");
      },
      { timeout: 2000 },
    );
  });
});
