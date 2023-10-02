/**
 * @jest-environment node
 */
import * as puppeteer from "puppeteer";
import { ChildProcess } from "child_process";

import {
  approveTransaction,
  approveConnection,
  createAccount,
  importAccount,
  transferFromTransparent,
} from "./partial";
import {
  launchPuppeteer,
  openPopup,
  setupNamada,
  startNamada,
  stopNamada,
  waitForInputValue,
  waitForXpath,
} from "./utils/helpers";
import {
  address0Alias,
  address1,
  shieldedAddress0,
  shieldedAddress0Alias,
} from "./utils/values";

jest.setTimeout(240000);

let browser: puppeteer.Browser;
let page: puppeteer.Page;

describe("Namada extension", () => {
  const namRefs = new Set<ChildProcess>();

  beforeEach(async function () {
    await setupNamada();
    browser = await launchPuppeteer();
    [page] = await browser.pages();
  });

  afterEach(async function () {
    // await browser.close();
  });

  afterAll(async () => {
    for await (const namada of namRefs) {
      // We want to stop the namada process only if:
      // - process is NOT about to stop - SIGKILL
      // - process is NOT already stopped - exitCode is not set
      const stop = !(
        namada.signalCode === "SIGKILL" || namada.exitCode !== null
      );
      if (stop) {
        await stopNamada(namada);
      }
    }
  });

  describe("open the popup", () => {
    test("should open the popup", async () => {
      await openPopup(browser, page);
      // Check H1
      const h1 = await page.$eval("h1", (e) => e.innerText);
      expect(h1).toEqual("Namada Browser Extension");
    });
  });

  describe("account", () => {
    test("create account & derive transparent address", async () => {
      await createAccount(browser, page);

      // Check if address was added
      openPopup(browser, page);
      await page.waitForNavigation();

      const addresses = await page.$$("li[class*='AccountsListItem']");

      expect(addresses.length).toEqual(1);

      // Click to derive new address
      await page.$eval(
        "div[class*='AccountListingContainer-'] a[class*='Button']",
        (e) => e.click()
      );

      // Derive new address
      const input = await page.$("input");
      input?.type(address0Alias);
      await waitForInputValue(page, input, address0Alias);

      (
        await waitForXpath<HTMLButtonElement>(
          page,
          "//button[contains(., 'Add')]"
        )
      ).click();

      // Check if address was added
      await page.waitForSelector("ul[class*='AccountsList']");
      const itemsLength = await page.$$eval(
        "li[class*='AccountsListItem']",
        (e) => e.length
      );

      expect(itemsLength).toEqual(2);
    });

    test("create account & derive shielded address", async () => {
      await createAccount(browser, page);

      // Check if address was added
      openPopup(browser, page);
      await page.waitForNavigation();

      const addresses = await page.$$("li[class*='AccountsListItem']");

      expect(addresses.length).toEqual(1);

      // Click to derive new address
      await page.$eval(
        "div[class*='AccountListingContainer-'] a[class*='Button']",
        (e) => e.click()
      );

      // Input text and wait
      const input = await page.$("input");
      input?.type(shieldedAddress0Alias);
      await waitForInputValue(page, input, shieldedAddress0Alias);

      // Switch to shielded
      page.$eval("button[data-testid='Toggle']", (e) => e.click());

      // Derive new address
      (
        await waitForXpath<HTMLButtonElement>(
          page,
          "//button[contains(., 'Add')]"
        )
      ).click();

      // Check if address was added
      await page.waitForSelector("ul[class*='AccountsList']");
      const itemsLength = await page.$$eval(
        "li[class*='AccountsListItem']",
        (e) => e.length
      );

      expect(itemsLength).toEqual(2);
    });
  });

  describe("transfer", () => {
    test("(transparent->transparent)", async () => {
      const nam = startNamada(namRefs);

      await importAccount(browser, page);
      await approveConnection(browser, page);

      await transferFromTransparent(browser, page, {
        targetAddress: address1,
      });

      await stopNamada(nam);
    });

    test("(transparent->shielded)", async () => {
      const nam = startNamada(namRefs);

      await importAccount(browser, page);
      await approveConnection(browser, page);

      await transferFromTransparent(browser, page, {
        targetAddress: shieldedAddress0,
        transferTimeout: 120000,
      });

      await stopNamada(nam);
    });
  });

  describe("staking", () => {
    test("bond and unbond", async () => {
      const nam = startNamada(namRefs);

      await importAccount(browser, page);
      await approveConnection(browser, page);

      // Click on send button
      (
        await waitForXpath<HTMLButtonElement>(
          page,
          "//button[contains(., 'Staking')]"
        )
      ).click();

      // Click on validator
      (
        await waitForXpath<HTMLSpanElement>(
          page,
          "//span[contains(., 'atest1v4')]"
        )
      ).click();

      // Click on stake button
      (
        await waitForXpath<HTMLButtonElement>(
          page,
          "//button[contains(., 'Stake')]"
        )
      ).click();

      // Type staking amount
      const [stakeInput] = await page.$$("input");
      await stakeInput.type("100");

      // Click confirm
      (
        await waitForXpath<HTMLButtonElement>(
          page,
          "//button[contains(., 'Confirm')]"
        )
      ).click();

      await approveTransaction(browser);

      // Wait for success toast
      const bondCompletedToast = await page.waitForXPath(
        "//div[contains(., 'Transaction completed!')]"
      );

      expect(bondCompletedToast).toBeDefined();

      // Click on unstake
      (
        await waitForXpath<HTMLSpanElement>(
          page,
          "//span[contains(., 'unstake')]"
        )
      ).click();

      await page.waitForNavigation();
      const [unstakeInput] = await page.$$("input");
      await unstakeInput.type("100");

      // Click confirm
      (
        await waitForXpath<HTMLButtonElement>(
          page,
          "//button[contains(., 'Confirm')]"
        )
      ).click();

      await approveTransaction(browser);

      // Wait for success toast
      const unbondCompletedToast = await page.waitForXPath(
        "//div[contains(., 'Transaction completed!')]"
      );

      expect(unbondCompletedToast).toBeDefined();

      await stopNamada(nam);
    });
  });
});
