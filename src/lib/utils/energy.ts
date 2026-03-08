import type { Track } from "$lib/types";

export function blendEnergy(track: Track): number[] | null {
  if (!track.energy_rms || !track.energy_centroid || !track.energy_onset) {
    return null;
  }
  return track.energy_rms.map(
    (r, i) =>
      0.5 * r + 0.3 * track.energy_centroid![i] + 0.2 * track.energy_onset![i],
  );
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
