import { expect, test } from '@playwright/test'

test('analyst can create an investigation', async ({ page }) => {
  const title = `E2E investigation ${Date.now()}`
  await page.goto('/')
  await page.getByLabel('Password').fill('jocky-demo')
  await page.getByRole('button', { name: 'Sign in' }).click()
  await expect(page.getByRole('heading', { name: 'Overview' })).toBeVisible()

  await page.getByRole('button', { name: 'New investigation' }).click()
  const dialog = page.getByRole('dialog', { name: 'investigation' })
  await dialog.getByLabel('Title').fill(title)
  await dialog.getByLabel('Description').fill('Authorized endpoint review')
  await dialog.getByRole('button', { name: 'Create' }).click()
  await expect(page.getByText(title)).toBeVisible()

  await page.getByRole('button', { name: 'Investigations' }).click()
  await expect(page.getByRole('table').getByText(title)).toBeVisible()
  await page.screenshot({ path: 'test-results/investigations-desktop.png', fullPage: true })

  await page.setViewportSize({ width: 390, height: 844 })
  await expect(page.getByRole('heading', { name: 'Investigations' })).toBeVisible()
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(390)
  await page.screenshot({ path: 'test-results/investigations-mobile.png', fullPage: true })
})
