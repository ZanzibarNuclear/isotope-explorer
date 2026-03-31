<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { PERIODIC_TABLE, type ElementPosition } from '../data/periodic-table-layout'

interface NuclideKey { z: number; n: number }
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

const props = defineProps<{
  session: any
}>()

const emit = defineEmits<{
  (e: 'select-isotope', z: number, n: number): void
}>()

// Build the nuclide index from the WASM session
const isotopesByElement = ref<Map<number, NuclideInfo[]>>(new Map())
const elementsWithData = ref<Set<number>>(new Set())
const selectedZ = ref<number | null>(null)
const selectedIsotope = ref<{ z: number; n: number } | null>(null)

function buildIndex() {
  if (!props.session) return
  const keys: NuclideKey[] = props.session.all_nuclide_keys()
  const byElement = new Map<number, NuclideInfo[]>()

  for (const { z, n } of keys) {
    const data = props.session.lookup(z, n)
    if (!data) continue

    const info: NuclideInfo = {
      z,
      n,
      a: z + n,
      is_stable: data.is_stable,
      is_fissile: data.is_fissile,
      half_life_s: data.half_life_s,
      notation: data.notation,
      decay_modes: data.decay_modes ?? [],
    }

    if (!byElement.has(z)) byElement.set(z, [])
    byElement.get(z)!.push(info)
  }

  // Sort isotopes by mass number
  for (const [, isotopes] of byElement) {
    isotopes.sort((a, b) => a.a - b.a)
  }

  isotopesByElement.value = byElement
  elementsWithData.value = new Set(byElement.keys())
}

watch(() => props.session, buildIndex, { immediate: true })

const selectedIsotopes = computed(() => {
  if (selectedZ.value === null) return []
  return isotopesByElement.value.get(selectedZ.value) ?? []
})

const selectedElementInfo = computed(() => {
  if (selectedZ.value === null) return null
  return PERIODIC_TABLE.find(e => e.z === selectedZ.value) ?? null
})

function selectElement(el: ElementPosition) {
  if (!elementsWithData.value.has(el.z)) return
  selectedZ.value = el.z
}

function selectIsotope(iso: NuclideInfo) {
  selectedIsotope.value = { z: iso.z, n: iso.n }
  emit('select-isotope', iso.z, iso.n)
}

function hasData(z: number): boolean {
  return elementsWithData.value.has(z)
}

function hasFissile(z: number): boolean {
  const isotopes = isotopesByElement.value.get(z)
  return isotopes?.some(i => i.is_fissile) ?? false
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
  <div class="picker">
    <!-- Periodic Table Grid -->
    <div class="pt-grid">
      <div
        v-for="el in PERIODIC_TABLE"
        :key="el.z"
        class="pt-cell"
        :class="{
          'has-data': hasData(el.z),
          'no-data': !hasData(el.z),
          'selected': selectedZ === el.z,
          'fissile': hasFissile(el.z),
          'lanthanide': el.category === 'lanthanide',
          'actinide': el.category === 'actinide',
        }"
        :style="{
          gridRow: el.row,
          gridColumn: el.col,
        }"
        :title="hasData(el.z)
          ? `${el.name} (Z=${el.z}) - ${isotopesByElement.get(el.z)?.length ?? 0} isotopes`
          : `${el.name} (Z=${el.z}) - no data`"
        @click="selectElement(el)"
      >
        <span class="pt-z">{{ el.z }}</span>
        <span class="pt-symbol">{{ el.symbol }}</span>
      </div>
      <!-- Lanthanide/actinide labels -->
      <div class="pt-label" :style="{ gridRow: 6, gridColumn: 3 }">*</div>
      <div class="pt-label" :style="{ gridRow: 7, gridColumn: 3 }">**</div>
      <div class="pt-series-label" :style="{ gridRow: 8, gridColumn: 1 }">*</div>
      <div class="pt-series-label" :style="{ gridRow: 9, gridColumn: 1 }">**</div>
    </div>

    <!-- Isotope row for selected element -->
    <div v-if="selectedZ !== null && selectedElementInfo" class="isotope-section">
      <div class="isotope-header">
        <span class="isotope-element-name">
          {{ selectedElementInfo.name }}
        </span>
        <span class="isotope-count">
          {{ selectedIsotopes.length }} isotope{{ selectedIsotopes.length !== 1 ? 's' : '' }}
        </span>
      </div>
      <div class="isotope-row">
        <button
          v-for="iso in selectedIsotopes"
          :key="iso.a"
          class="isotope-btn"
          :class="{
            stable: iso.is_stable,
            fissile: iso.is_fissile,
            radioactive: !iso.is_stable && !iso.is_fissile,
            selected: selectedIsotope?.z === iso.z && selectedIsotope?.n === iso.n,
          }"
          :title="`${iso.notation}\n${iso.is_stable ? 'Stable' : formatHalfLife(iso.half_life_s)}`"
          @click="selectIsotope(iso)"
        >
          {{ iso.a }}
        </button>
      </div>
    </div>
    <div v-else class="isotope-section isotope-placeholder">
      Click an element to see its isotopes
    </div>
  </div>
</template>

<style scoped>
.picker {
  padding: 0.75rem 1.25rem;
  border-bottom: 1px solid #30363d;
  background: #0d1117;
}

/* ---- Periodic Table Grid ---- */
.pt-grid {
  display: grid;
  grid-template-columns: repeat(18, 1fr);
  grid-template-rows: repeat(9, auto);
  gap: 1px;
  max-width: 900px;
  margin: 0 auto;
  row-gap: 1px;
}

/* Gap between main table and lanthanide/actinide rows */
.pt-grid > *[style*="grid-row: 8"],
.pt-grid > *[style*="grid-row: 9"] {
  margin-top: 4px;
}

.pt-cell {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 2px 1px;
  border-radius: 3px;
  cursor: default;
  transition: background 0.12s, border-color 0.12s;
  border: 1px solid transparent;
  min-width: 0;
  aspect-ratio: 1;
  position: relative;
}

.pt-cell.has-data {
  background: #21262d;
  cursor: pointer;
}
.pt-cell.has-data:hover {
  background: #30363d;
  border-color: #58a6ff;
}
.pt-cell.no-data {
  background: #161b2266;
  opacity: 0.35;
}
.pt-cell.selected {
  background: #1f6feb33;
  border-color: #58a6ff;
}
.pt-cell.fissile.has-data {
  background: #f0883e22;
}
.pt-cell.fissile.has-data:hover {
  background: #f0883e44;
}
.pt-cell.fissile.selected {
  background: #f0883e33;
  border-color: #f0883e;
}
.pt-cell.actinide.has-data:not(.fissile):not(.selected) {
  background: #2d1f3d;
}
.pt-cell.lanthanide.has-data:not(.selected) {
  background: #1f2d3d;
}

.pt-z {
  font-size: 0.5rem;
  color: #8b949e;
  line-height: 1;
}
.pt-symbol {
  font-size: 0.75rem;
  font-weight: 600;
  line-height: 1.2;
}

.pt-label {
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.7rem;
  color: #484f58;
}
.pt-series-label {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  padding-right: 2px;
  font-size: 0.7rem;
  color: #484f58;
  grid-column: 1 / 3;
}

/* ---- Isotope Row ---- */
.isotope-section {
  margin-top: 0.75rem;
  min-height: 2.5rem;
}
.isotope-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  color: #484f58;
  font-size: 0.85rem;
}
.isotope-header {
  display: flex;
  align-items: baseline;
  gap: 0.5rem;
  margin-bottom: 0.4rem;
}
.isotope-element-name {
  font-weight: 600;
  font-size: 0.95rem;
}
.isotope-count {
  font-size: 0.8rem;
  color: #8b949e;
}
.isotope-row {
  display: flex;
  flex-wrap: wrap;
  gap: 3px;
}
.isotope-btn {
  padding: 0.25rem 0.45rem;
  border: 1px solid #30363d;
  border-radius: 4px;
  background: #21262d;
  color: #e6edf3;
  font-size: 0.8rem;
  font-weight: 500;
  cursor: pointer;
  transition: border-color 0.12s, background 0.12s;
  min-width: 2.2rem;
  text-align: center;
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
</style>
