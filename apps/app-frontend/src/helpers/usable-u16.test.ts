import assert from 'node:assert/strict'
import test from 'node:test'

import { usableU16 } from './usable-u16.ts'

test('accepts integers inside the u16 window range', () => {
    assert.equal(usableU16(1), 1)
    assert.equal(usableU16(1280), 1280)
    assert.equal(usableU16(65535), 65535)
})

test('rejects everything the u16 window field cannot hold', () => {
    const rejected = [
        0,
        -1,
        65536,
        1920.5,
        Number.NaN,
        Number.POSITIVE_INFINITY,
        undefined,
        null,
        '1280',
        true,
    ]

    for (const value of rejected) {
        assert.equal(usableU16(value), null, `expected ${String(value)} to be rejected`)
    }
})
