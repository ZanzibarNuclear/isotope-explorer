export interface QuickPickIsotope {
  z: number
  a: number
  label?: string
  primaryUse: string
  stable?: boolean
}

export interface QuickPickCategory {
  name: string
  isotopes: QuickPickIsotope[]
}

export const QUICK_PICK_CATEGORIES: QuickPickCategory[] = [
  {
    name: 'Fission fuels',
    isotopes: [
      { z: 92, a: 233, primaryUse: 'Nuclear reactor fuel' },
      { z: 92, a: 235, primaryUse: 'Nuclear reactor fuel' },
      { z: 92, a: 238, primaryUse: 'Fertile breeder material' },
      { z: 94, a: 238, primaryUse: 'Radioisotope power source' },
      { z: 94, a: 239, primaryUse: 'Nuclear reactor fuel' },
      { z: 94, a: 241, primaryUse: 'Nuclear reactor fuel' },
      { z: 90, a: 232, primaryUse: 'Fertile breeder material' },
    ],
  },
  {
    name: 'Hydrogen family',
    isotopes: [
      { z: 1, a: 1, primaryUse: 'Ordinary hydrogen', stable: true },
      { z: 1, a: 2, primaryUse: 'Heavy water and isotope tracing', stable: true },
      { z: 1, a: 3, primaryUse: 'Radioluminescent devices' },
      { z: 2, a: 3, primaryUse: 'Neutron detection', stable: true },
      { z: 2, a: 4, primaryUse: 'Cryogenics and inert gas', stable: true },
      { z: 3, a: 6, primaryUse: 'Tritium production' },
      { z: 3, a: 7, primaryUse: 'Reactor coolant chemistry' },
    ],
  },
  {
    name: 'Common radioisotopes',
    isotopes: [
      { z: 6, a: 12, primaryUse: 'Stable carbon reference', stable: true },
      { z: 6, a: 14, primaryUse: 'Radiocarbon dating' },
      { z: 27, a: 59, primaryUse: 'Cobalt activation target', stable: true },
      { z: 27, a: 60, primaryUse: 'Gamma irradiation source' },
      { z: 38, a: 88, primaryUse: 'Stable strontium reference', stable: true },
      { z: 38, a: 90, primaryUse: 'Beta source' },
      { z: 53, a: 127, primaryUse: 'Stable iodine reference', stable: true },
      { z: 53, a: 131, primaryUse: 'Thyroid therapy and imaging' },
      { z: 55, a: 133, primaryUse: 'Atomic clocks', stable: true },
      { z: 55, a: 137, primaryUse: 'Gamma source' },
    ],
  },
  {
    name: 'Medicine',
    isotopes: [
      { z: 43, a: 99, label: 'Tc-99m', primaryUse: 'Diagnostic imaging' },
      { z: 42, a: 99, primaryUse: 'Tc-99m generator parent' },
      { z: 53, a: 131, primaryUse: 'Thyroid therapy' },
      { z: 53, a: 125, primaryUse: 'Brachytherapy' },
      { z: 9, a: 18, primaryUse: 'PET imaging' },
      { z: 27, a: 60, primaryUse: 'Radiotherapy and sterilization' },
      { z: 71, a: 177, primaryUse: 'Targeted radionuclide therapy' },
      { z: 89, a: 225, primaryUse: 'Alpha therapy' },
      { z: 39, a: 90, primaryUse: 'Radioembolization therapy' },
    ],
  },
  {
    name: 'Stable',
    isotopes: [
      { z: 6, a: 12, primaryUse: 'Stable isotope reference', stable: true },
      { z: 7, a: 15, primaryUse: 'Stable isotope tracer', stable: true },
      { z: 8, a: 18, primaryUse: 'Stable isotope tracer', stable: true },
      { z: 1, a: 1, primaryUse: 'Ordinary hydrogen', stable: true },
      { z: 2, a: 3, primaryUse: 'Neutron detection', stable: true },
      { z: 2, a: 4, primaryUse: 'Cryogenics and inert gas', stable: true },
    ],
  },
  {
    name: 'Everyday Use',
    isotopes: [
      { z: 95, a: 241, primaryUse: 'Smoke detectors' },
      { z: 27, a: 60, primaryUse: 'Sterilization and radiography' },
      { z: 55, a: 137, primaryUse: 'Industrial gauges' },
      { z: 77, a: 192, primaryUse: 'Industrial radiography' },
      { z: 1, a: 3, primaryUse: 'Radioluminescent devices' },
      { z: 98, a: 252, primaryUse: 'Neutron source' },
    ],
  },
  {
    name: 'Tracing and Dating',
    isotopes: [
      { z: 6, a: 14, primaryUse: 'Radiocarbon dating' },
      { z: 1, a: 2, primaryUse: 'Water tracing', stable: true },
      { z: 15, a: 32, primaryUse: 'Biochemical tracer' },
      { z: 15, a: 33, primaryUse: 'DNA and RNA labeling' },
      { z: 16, a: 35, primaryUse: 'Protein and metabolic tracing' },
      { z: 24, a: 51, primaryUse: 'Blood volume tracing' },
      { z: 7, a: 15, primaryUse: 'Nitrogen cycle tracing', stable: true },
      { z: 8, a: 18, primaryUse: 'Water and metabolic tracing', stable: true },
    ],
  },
]
