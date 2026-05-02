<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { PERIODIC_TABLE } from '../data/periodic-table-layout'
import { QUICK_PICK_CATEGORIES, type QuickPickIsotope } from '../data/quick-picks'

const elementNameByZ = new Map(PERIODIC_TABLE.map(e => [e.z, e.name]))
const elementSymbolByZ = new Map(PERIODIC_TABLE.map(e => [e.z, e.symbol]))
const elementName = (z: number) => elementNameByZ.get(z) ?? 'Unknown'

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

interface QuickPickRow extends NuclideInfo {
  primaryUse: string
}

const isotopesByElement = ref<Map<number, NuclideInfo[]>>(new Map())
const selectedIsotope = ref<{ z: number; n: number } | null>(null)
const selectedCategoryName = ref(QUICK_PICK_CATEGORIES[0]?.name ?? '')
const selectedCategory = computed(() =>
  QUICK_PICK_CATEGORIES.find(category => category.name === selectedCategoryName.value) ?? QUICK_PICK_CATEGORIES[0]!
)

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

function resolveIsotope(pick: QuickPickIsotope): QuickPickRow {
  const all = isotopesByElement.value.get(pick.z) ?? []
  const found = all.find(iso => iso.a === pick.a)
  const notation = pick.label ?? found?.notation ?? `${elementSymbolByZ.get(pick.z) ?? elementName(pick.z)}-${pick.a}`

  return {
    z: pick.z,
    n: found?.n ?? Math.max(0, pick.a - pick.z),
    a: pick.a,
    is_stable: pick.stable ?? found?.is_stable ?? false,
    is_fissile: found?.is_fissile ?? false,
    half_life_s: found?.half_life_s ?? null,
    notation,
    decay_modes: found?.decay_modes ?? [],
    primaryUse: pick.primaryUse,
  }
}

const selectedRows = computed(() => selectedCategory.value.isotopes.map(resolveIsotope))

function selectIsotope(iso: QuickPickRow) {
  selectedIsotope.value = { z: iso.z, n: iso.n }
  emit('select-isotope', iso.z, iso.n)
}

function isSelected(iso: QuickPickRow): boolean {
  return selectedIsotope.value?.z === iso.z && selectedIsotope.value?.n === iso.n
}
</script>

<template>
  <div class="quick-pick">
    <label class="category-select">
      <span class="category-label">Category</span>
      <select v-model="selectedCategoryName">
        <option v-for="category in QUICK_PICK_CATEGORIES" :key="category.name" :value="category.name">
          {{ category.name }}
        </option>
      </select>
    </label>

    <div class="group">
      <div class="group-label">{{ selectedCategory.name }}</div>
      <div class="isotope-table">
        <div class="table-header">
          <span>Isotope</span>
          <span>Element</span>
          <span>Primary use</span>
        </div>
        <div v-for="iso in selectedRows" :key="`${iso.z}-${iso.a}`" class="entry">
          <button
            class="isotope-btn"
            :class="{
              stable: iso.is_stable,
              fissile: iso.is_fissile,
              radioactive: !iso.is_stable && !iso.is_fissile,
              selected: isSelected(iso),
            }"
            :title="`${elementName(iso.z)}-${iso.a}`"
            @click="selectIsotope(iso)"
          >
            {{ iso.notation }}
          </button>
          <span class="entry-name">{{ elementName(iso.z) }}</span>
          <span class="primary-use">{{ iso.primaryUse }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.quick-pick {
  padding: 0.6rem 1.25rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  background: #0d1117;
}

.category-select {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.category-label {
  font-size: 0.75rem;
  font-weight: 600;
  color: #8b949e;
}

.category-select select {
  min-width: 12rem;
  padding: 0.25rem 0.5rem;
  border: 1px solid #30363d;
  border-radius: 4px;
  background: #161b22;
  color: #e6edf3;
  font: inherit;
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

.isotope-table {
  display: grid;
  gap: 0.25rem;
  max-width: 46rem;
}

.table-header,
.entry {
  display: grid;
  grid-template-columns: 6rem 8rem minmax(12rem, 1fr);
  align-items: center;
  gap: 0.75rem;
}

.table-header {
  font-size: 0.7rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: #484f58;
}

.entry {
  min-width: 0;
}

.entry-name {
  font-size: 0.8rem;
  color: #8b949e;
  min-width: 0;
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

.primary-use {
  min-width: 0;
  color: #c9d1d9;
  font-size: 0.8rem;
}

@media (max-width: 560px) {
  .table-header {
    display: none;
  }

  .entry {
    grid-template-columns: 5.5rem 1fr;
    gap: 0.25rem 0.75rem;
  }

  .primary-use {
    grid-column: 2;
  }
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
