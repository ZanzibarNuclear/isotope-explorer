<script setup lang="ts">
import { ref, watch } from 'vue'

interface NuclideInfo {
  z: number
  n: number
  a: number
  is_stable: boolean
  is_fissile: boolean
  half_life_s: number | null
  notation: string
  decay_modes: string[]
}

const props = defineProps<{ session: any }>()
const emit = defineEmits<{ (e: 'select-isotope', z: number, n: number): void }>()

// Curated groups of elements with interesting isotope progressions
const QUICK_PICKS: { group: string; entries: { z: number; label: string; highlight: number[] }[] }[] = [
  {
    group: 'Fission fuels',
    entries: [
      { z: 92, label: 'Uranium', highlight: [233, 235, 238] },
      { z: 94, label: 'Plutonium', highlight: [239, 241] },
      { z: 90, label: 'Thorium', highlight: [232] },
    ],
  },
  {
    group: 'Hydrogen family',
    entries: [
      { z: 1, label: 'Hydrogen', highlight: [1, 2, 3] },
      { z: 2, label: 'Helium', highlight: [3, 4] },
      { z: 3, label: 'Lithium', highlight: [6, 7] },
    ],
  },
  {
    group: 'Uranium decay chain',
    entries: [
      { z: 88, label: 'Radium', highlight: [226] },
      { z: 86, label: 'Radon', highlight: [222] },
      { z: 84, label: 'Polonium', highlight: [210, 214, 218] },
      { z: 83, label: 'Bismuth', highlight: [209, 210, 214] },
      { z: 82, label: 'Lead', highlight: [206, 207, 208] },
    ],
  },
  {
    group: 'Common radioisotopes',
    entries: [
      { z: 6, label: 'Carbon', highlight: [12, 14] },
      { z: 27, label: 'Cobalt', highlight: [59, 60] },
      { z: 38, label: 'Strontium', highlight: [88, 90] },
      { z: 53, label: 'Iodine', highlight: [127, 131] },
      { z: 55, label: 'Cesium', highlight: [133, 137] },
    ],
  },
]

const isotopesByElement = ref<Map<number, NuclideInfo[]>>(new Map())
const selectedIsotope = ref<{ z: number; n: number } | null>(null)

function buildIndex() {
  if (!props.session) return
  const keys = props.session.all_nuclide_keys()
  const byElement = new Map<number, NuclideInfo[]>()
  for (const { z, n } of keys) {
    const data = props.session.lookup(z, n)
    if (!data) continue
    const info: NuclideInfo = {
      z, n, a: z + n,
      is_stable: data.is_stable,
      is_fissile: data.is_fissile,
      half_life_s: data.half_life_s,
      notation: data.notation,
      decay_modes: data.decay_modes ?? [],
    }
    if (!byElement.has(z)) byElement.set(z, [])
    byElement.get(z)!.push(info)
  }
  for (const [, isotopes] of byElement) isotopes.sort((a, b) => a.a - b.a)
  isotopesByElement.value = byElement
}

watch(() => props.session, buildIndex, { immediate: true })

function getIsotopes(z: number, highlight: number[]): NuclideInfo[] {
  const all = isotopesByElement.value.get(z) ?? []
  const pinned = all.filter(iso => highlight.includes(iso.a))
  return pinned.length > 0 ? pinned : all.slice(0, 6)
}

function selectIsotope(iso: NuclideInfo) {
  selectedIsotope.value = { z: iso.z, n: iso.n }
  emit('select-isotope', iso.z, iso.n)
}

function isSelected(iso: NuclideInfo): boolean {
  return selectedIsotope.value?.z === iso.z && selectedIsotope.value?.n === iso.n
}

function formatHalfLife(seconds: number | null): string {
  if (seconds === null) return 'stable'
  if (seconds < 1e-15) return `${(seconds * 1e18).toFixed(1)} as`
  if (seconds < 1e-12) return `${(seconds * 1e15).toFixed(1)} fs`
  if (seconds < 1e-9) return `${(seconds * 1e12).toFixed(1)} ps`
  if (seconds < 1e-6) return `${(seconds * 1e9).toFixed(1)} ns`
  if (seconds < 1e-3) return `${(seconds * 1e6).toFixed(1)} \u00b5s`
  if (seconds < 1) return `${(seconds * 1e3).toFixed(1)} ms`
  if (seconds < 60) return `${seconds.toFixed(1)} s`
  if (seconds < 3600) return `${(seconds / 60).toFixed(1)} min`
  if (seconds < 86400) return `${(seconds / 3600).toFixed(1)} h`
  if (seconds < 365.25 * 86400) return `${(seconds / 86400).toFixed(1)} d`
  const years = seconds / (365.25 * 86400)
  if (years < 1e3) return `${years.toFixed(1)} y`
  if (years < 1e6) return `${(years / 1e3).toFixed(1)} ky`
  if (years < 1e9) return `${(years / 1e6).toFixed(1)} My`
  if (years < 1e12) return `${(years / 1e9).toFixed(2)} Gy`
  return `${(years / 1e12).toFixed(1)} Ty`
}
</script>

<template>
  <div class="quick-pick">
    <div v-for="group in QUICK_PICKS" :key="group.group" class="group">
      <div class="group-label">{{ group.group }}</div>
      <div class="group-entries">
        <div
          v-for="entry in group.entries"
          :key="entry.z"
          class="entry"
        >
          <span class="entry-name">{{ entry.label }}</span>
          <div class="entry-isotopes">
            <button
              v-for="iso in getIsotopes(entry.z, entry.highlight)"
              :key="iso.a"
              class="isotope-btn"
              :class="{
                stable: iso.is_stable,
                fissile: iso.is_fissile,
                radioactive: !iso.is_stable && !iso.is_fissile,
                selected: isSelected(iso),
              }"
              :title="`${iso.notation} — ${formatHalfLife(iso.half_life_s)}`"
              @click="selectIsotope(iso)"
            >
              {{ iso.notation }}
            </button>
            <span v-if="getIsotopes(entry.z, entry.highlight).length === 0" class="no-data">
              no data
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.quick-pick {
  padding: 0.6rem 1.25rem;
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem 1.5rem;
  background: #0d1117;
}

.group {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  min-width: 0;
}

.group-label {
  font-size: 0.7rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: #484f58;
}

.group-entries {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.entry {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.entry-name {
  font-size: 0.8rem;
  color: #8b949e;
  min-width: 5.5rem;
  flex-shrink: 0;
}

.entry-isotopes {
  display: flex;
  flex-wrap: wrap;
  gap: 3px;
}

.isotope-btn {
  padding: 0.2rem 0.4rem;
  border: 1px solid #30363d;
  border-radius: 4px;
  background: #21262d;
  color: #e6edf3;
  font-size: 0.8rem;
  font-weight: 500;
  cursor: pointer;
  transition: border-color 0.12s, background 0.12s;
  white-space: nowrap;
}
.isotope-btn:hover {
  border-color: #58a6ff;
  background: #1c2128;
}
.isotope-btn.selected {
  border-color: #58a6ff;
  background: #1f6feb33;
}
.isotope-btn.stable {
  border-left: 2px solid #3fb950;
}
.isotope-btn.fissile {
  border-left: 2px solid #f0883e;
  font-weight: 700;
}
.isotope-btn.radioactive {
  border-left: 2px solid #8b949e;
}

.no-data {
  font-size: 0.75rem;
  color: #484f58;
}
</style>
