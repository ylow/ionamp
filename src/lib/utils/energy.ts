import type { Track } from "$lib/types";

const blendCache = new WeakMap<Track, number[] | null>();

export function blendEnergy(track: Track): number[] | null {
  let cached = blendCache.get(track);
  if (cached !== undefined) return cached;

  let result: number[] | null = null;
  if (track.energy_rms && track.energy_centroid && track.energy_onset) {
    result = track.energy_rms.map(
      (r, i) =>
        0.5 * r + 0.3 * track.energy_centroid![i] + 0.2 * track.energy_onset![i],
    );
  }
  blendCache.set(track, result);
  return result;
}

export function maxEnergyOfTracks(tracks: Iterable<Track>): number {
  let max = 0;
  for (const track of tracks) {
    const blended = blendEnergy(track);
    if (blended) {
      for (const v of blended) {
        if (v > max) max = v;
      }
    }
  }
  return max;
}
