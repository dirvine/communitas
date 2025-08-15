import { featureFlags } from '../featureFlags'

describe('FeatureFlagsService', () => {
  beforeEach(() => {
    // @ts-ignore
    localStorage.clear()
  })

  it('enables and persists flags', () => {
    expect(featureFlags.isEnabled('unified-design-system')).toBe(false)
    featureFlags.enable('unified-design-system')
    expect(featureFlags.isEnabled('unified-design-system')).toBe(true)

    // simulate reload
    // @ts-ignore
    const saved = localStorage.getItem('communitas-feature-flags')!
    const parsed = JSON.parse(saved)
    expect(parsed['unified-design-system'].enabled).toBe(true)
  })

  it('honors rolloutPercentage for a user', () => {
    featureFlags.setRolloutPercentage('unified-design-system', 0)
    expect(featureFlags.isEnabled('unified-design-system', 'user_a')).toBe(false)
    featureFlags.setRolloutPercentage('unified-design-system', 100)
    expect(featureFlags.isEnabled('unified-design-system', 'user_b')).toBe(true)
  })
})
