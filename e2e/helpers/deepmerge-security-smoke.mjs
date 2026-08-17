import assert from 'node:assert/strict'

import { deepmerge, deepmergeCustom } from 'deepmerge-ts'

// WebdriverIO 9.30.1 uses this exact deepmergeCustom surface in ConfigParser:
// mergeArrays receives (values, utils, meta), inspects meta.key, and may return
// utils.actions.defaultMerge. Keep this smoke focused on that compatibility
// boundary while the root override forces the security-fixed deepmerge-ts.
const mergeConfig = deepmergeCustom({
  mergeArrays: ([oldValue, newValue], utils, meta) => {
    if (meta?.key === 'services') {
      const primitiveOldValues = oldValue.filter((value) => typeof value !== 'object')
      return Array.from(new Set(deepmerge(newValue, primitiveOldValues)))
    }
    return utils.actions.defaultMerge
  }
})

const mergedConfig = mergeConfig(
  {
    services: ['legacy-service', ['legacy-service-object', { enabled: true }]],
    specs: ['legacy.e2e.js']
  },
  {
    services: ['current-service'],
    specs: ['current.e2e.js']
  }
)

assert.deepEqual(mergedConfig.services, ['current-service', 'legacy-service'])
assert.deepEqual(mergedConfig.specs, ['legacy.e2e.js', 'current.e2e.js'])

// GHSA-ggr8-5vv4-36mx affects recursive object graphs. deepmerge-ts 8.0.0
// adds circular-reference support; prove the resolved runtime no longer takes
// the vulnerable stack-exhaustion path for the public API used upstream.
const left = { left: true }
left.self = left
const right = { right: true }
right.self = right
const mergedCircular = deepmerge(left, right)

assert.equal(mergedCircular.left, true)
assert.equal(mergedCircular.right, true)
assert.equal(mergedCircular.self, mergedCircular)

console.log('deepmerge security compatibility smoke: PASS')
