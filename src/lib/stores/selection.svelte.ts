class SelectionState {
  selectedIds = $state<Set<number>>(new Set());
  lastSelectedId = $state<number | null>(null);

  select(id: number) {
    this.selectedIds = new Set([id]);
    this.lastSelectedId = id;
  }

  toggle(id: number) {
    const next = new Set(this.selectedIds);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    this.selectedIds = next;
    this.lastSelectedId = id;
  }

  rangeSelect(id: number, allIds: number[]) {
    if (this.lastSelectedId === null) {
      this.select(id);
      return;
    }

    const startIdx = allIds.indexOf(this.lastSelectedId);
    const endIdx = allIds.indexOf(id);
    if (startIdx === -1 || endIdx === -1) {
      this.select(id);
      return;
    }

    const from = Math.min(startIdx, endIdx);
    const to = Math.max(startIdx, endIdx);
    const rangeIds = allIds.slice(from, to + 1);

    const next = new Set(this.selectedIds);
    for (const rid of rangeIds) {
      next.add(rid);
    }
    this.selectedIds = next;
    this.lastSelectedId = id;
  }

  isSelected(id: number): boolean {
    return this.selectedIds.has(id);
  }

  clear() {
    this.selectedIds = new Set();
    this.lastSelectedId = null;
  }

  get count(): number {
    return this.selectedIds.size;
  }

  get ids(): number[] {
    return Array.from(this.selectedIds);
  }
}

export const selectionState = new SelectionState();
