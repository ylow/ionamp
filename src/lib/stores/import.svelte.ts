import { importFiles } from "$lib/api";
import type { ImportEvent } from "$lib/types";

class ImportState {
  importing = $state(false);
  current = $state(0);
  total = $state(0);
  currentFile = $state("");
  imported = $state(0);
  skipped = $state(0);
  errors = $state(0);

  get progress(): number {
    return this.total > 0 ? this.current / this.total : 0;
  }

  async runImport(paths: string[]) {
    this.importing = true;
    this.current = 0;
    this.total = 0;
    this.imported = 0;
    this.skipped = 0;
    this.errors = 0;

    try {
      await importFiles(paths, (event: ImportEvent) => {
        switch (event.type) {
          case "ScanComplete":
            this.total = event.total_files ?? 0;
            break;
          case "Progress":
            this.current = event.current ?? 0;
            this.total = event.total ?? 0;
            this.currentFile = event.file_name ?? "";
            break;
          case "Complete":
            this.imported = event.imported ?? 0;
            this.skipped = event.skipped ?? 0;
            this.errors = event.errors ?? 0;
            break;
        }
      });
    } catch (e) {
      console.error("Import error:", e);
    } finally {
      this.importing = false;
    }
  }
}

export const importState = new ImportState();
