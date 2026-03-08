import { searchTracks, getFilterOptions, clusterByEnergy } from "$lib/api";
import { maxEnergyOfTracks } from "$lib/utils/energy";
import type {
  SearchQuery,
  SearchResult,
  FilterOptions,
  FilterPredicate,
  GroupByField,
  TrackGroup,
  EnergyClusterGroup,
} from "$lib/types";

class SearchState {
  text = $state("");
  filters = $state<FilterPredicate[]>([]);
  groupBy = $state<GroupByField | "Energy" | null>(null);
  clusterSeed = $state(42);
  results = $state<TrackGroup[]>([]);
  totalCount = $state(0);
  maxEnergy = $state(0);
  loading = $state(false);
  filterOptions = $state<FilterOptions>({
    artists: [],
    albums: [],
    genres: [],
    years: [],
  });

  private debounceTimer: ReturnType<typeof setTimeout> | null = null;

  async search() {
    this.loading = true;
    try {
      if (this.groupBy === "Energy") {
        const query: SearchQuery = {
          text: this.text || null,
          filters: this.filters,
          group_by: null,
          limit: null,
          offset: null,
        };
        const clusters: EnergyClusterGroup[] = await clusterByEnergy(query, this.clusterSeed);
        this.results = clusters.map((c) => ({
          key: c.label,
          tracks: c.tracks,
        }));
        this.totalCount = clusters.reduce(
          (sum, c) => sum + c.tracks.length,
          0,
        );
      } else {
        const query: SearchQuery = {
          text: this.text || null,
          filters: this.filters,
          group_by: (this.groupBy as GroupByField) ?? null,
          limit: null,
          offset: null,
        };
        const result: SearchResult = await searchTracks(query);
        this.results = result.groups;
        this.totalCount = result.total_count;
      }
      this.maxEnergy = maxEnergyOfTracks(
        this.results.flatMap((g) => g.tracks),
      );
    } catch (e) {
      console.error("Search error:", e);
    } finally {
      this.loading = false;
    }
  }

  debouncedSearch() {
    if (this.debounceTimer) clearTimeout(this.debounceTimer);
    this.debounceTimer = setTimeout(() => this.search(), 50);
  }

  async loadFilterOptions() {
    try {
      this.filterOptions = await getFilterOptions();
    } catch (e) {
      console.error("Failed to load filter options:", e);
    }
  }

  setFilter(predicate: FilterPredicate) {
    this.filters = [...this.filters, predicate];
    this.search();
  }

  removeFilter(index: number) {
    this.filters = this.filters.filter((_, i) => i !== index);
    this.search();
  }

  clearFilters() {
    this.filters = [];
    this.search();
  }

  setGroupBy(group: GroupByField | "Energy" | null) {
    this.groupBy = group;
    this.search();
  }
}

export const searchState = new SearchState();
