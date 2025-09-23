#!/usr/bin/env python3
"""
Test the complete authentication flow: logout current user, then create new user.
"""

import time
import json
import random
from datetime import datetime
from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
from selenium.common.exceptions import NoSuchElementException, TimeoutException

def generate_test_identity():
    """Generate a test identity with four-word address."""
    words = ['ocean', 'forest', 'mountain', 'river', 'star', 'moon', 'sun', 'wind',
             'lake', 'tree', 'cloud', 'sky']
    four_words = '-'.join(random.sample(words, 4))
    display_name = f"TestUser_{random.randint(1000, 9999)}"
    return four_words, display_name

def test_full_auth_flow():
    """Test logout and new user creation flow."""

    # Generate test data
    four_words, display_name = generate_test_identity()
    device_name = "Test Device"

    print(f"\n🧪 Testing Full Authentication Flow")
    print(f"   Four Words: {four_words}")
    print(f"   Display Name: {display_name}")

    # Configure Chrome options
    options = webdriver.ChromeOptions()
    options.add_argument('--no-sandbox')
    options.add_argument('--disable-dev-shm-usage')
    options.add_argument('--disable-gpu')
    options.add_experimental_option('excludeSwitches', ['enable-logging'])

    driver = None
    results = {
        'timestamp': datetime.now().isoformat(),
        'test_data': {
            'four_words': four_words,
            'display_name': display_name,
            'device_name': device_name
        },
        'steps': []
    }

    try:
        # Initialize driver
        print("\n1️⃣ Starting Chrome driver...")
        driver = webdriver.Chrome(options=options)
        driver.set_window_size(1280, 800)
        wait = WebDriverWait(driver, 10)

        # Navigate to app
        print("2️⃣ Navigating to app...")
        driver.get("http://localhost:1422")
        time.sleep(3)  # Wait for app to load

        # Clear localStorage to ensure clean state
        print("3️⃣ Clearing cached authentication...")
        driver.execute_script("""
            localStorage.clear();
            sessionStorage.clear();
            // Clear IndexedDB if it exists
            if (window.indexedDB) {
                indexedDB.databases().then(databases => {
                    databases.forEach(db => {
                        indexedDB.deleteDatabase(db.name);
                    });
                });
            }
        """)
        time.sleep(1)

        # Refresh page to apply clean state
        driver.refresh()
        time.sleep(3)

        driver.save_screenshot("test_1_after_clear.png")
        print("   ✅ Cleared authentication cache")

        # Check current state - are we logged in?
        is_logged_in = False
        try:
            # Look for avatar or user indicator
            avatar = driver.find_element(By.CSS_SELECTOR, "[class*='Avatar']")
            if avatar.is_displayed():
                is_logged_in = True
                print("4️⃣ User is currently logged in, need to logout...")
        except:
            print("4️⃣ No user logged in, proceeding to create identity...")

        if is_logged_in:
            # Click on user avatar to open menu
            try:
                avatar = driver.find_element(By.CSS_SELECTOR, "[class*='Avatar']")
                avatar.click()
                time.sleep(1)

                # Look for Sign Out option
                sign_out = driver.find_element(By.XPATH, "//li[contains(., 'Sign Out')]")
                sign_out.click()
                time.sleep(2)
                print("   ✅ Logged out successfully")
                driver.save_screenshot("test_2_after_logout.png")
            except Exception as e:
                print(f"   ⚠️ Could not logout: {e}")

        # Now look for Sign In button
        print("\n5️⃣ Looking for Sign In button...")
        sign_in_button = None

        # Try multiple selectors
        selectors = [
            "//button[contains(text(), 'Sign In')]",
            "//button[contains(., 'Sign In')]",
            "//button[contains(text(), 'SIGN IN')]",
            "//button[contains(@aria-label, 'Sign')]"
        ]

        for selector in selectors:
            try:
                buttons = driver.find_elements(By.XPATH, selector)
                for btn in buttons:
                    if btn.is_displayed() and btn.is_enabled():
                        sign_in_button = btn
                        print(f"   Found button: {btn.text}")
                        break
                if sign_in_button:
                    break
            except:
                pass

        if not sign_in_button:
            # Try looking for any button with "sign" text
            buttons = driver.find_elements(By.TAG_NAME, "button")
            for btn in buttons:
                text = btn.text.lower()
                if "sign" in text and "in" in text:
                    sign_in_button = btn
                    print(f"   Found button by text search: {btn.text}")
                    break

        if sign_in_button:
            print("6️⃣ Clicking Sign In button...")

            # Scroll to button
            driver.execute_script("arguments[0].scrollIntoView(true);", sign_in_button)
            time.sleep(0.5)

            # Try regular click first
            try:
                sign_in_button.click()
            except:
                # Try JavaScript click
                driver.execute_script("arguments[0].click();", sign_in_button)

            time.sleep(2)
            driver.save_screenshot("test_3_after_signin_click.png")

            # Check if dialog opened
            dialog = None
            try:
                dialog = wait.until(EC.presence_of_element_located((By.CSS_SELECTOR, "[role='dialog']")))
                print("   ✅ Login dialog opened!")
            except TimeoutException:
                print("   ❌ Login dialog did not open")
                # Check console for errors
                logs = driver.get_log('browser')
                for log in logs:
                    if log['level'] == 'SEVERE':
                        print(f"   Console error: {log['message']}")

            if dialog:
                # Look for Create Identity option
                print("\n7️⃣ Switching to Create Identity mode...")

                create_buttons = [
                    "//button[contains(text(), 'Create Identity')]",
                    "//button[contains(text(), 'Create')]",
                    "//div[@role='tab'][contains(., 'Create')]",
                    "//button[@role='tab'][contains(., 'Create')]"
                ]

                for selector in create_buttons:
                    try:
                        create_btn = driver.find_element(By.XPATH, selector)
                        if create_btn.is_displayed():
                            create_btn.click()
                            time.sleep(1)
                            print("   ✅ Switched to Create mode")
                            break
                    except:
                        pass

                # Fill the form
                print("\n8️⃣ Filling identity form...")

                # Try different input selectors
                inputs_filled = 0

                # Four words
                for selector in ["//input[contains(@placeholder, 'four')]", "//input[contains(@name, 'fourWords')]", "//input[contains(@id, 'fourWords')]"]:
                    try:
                        input_field = driver.find_element(By.XPATH, selector)
                        input_field.clear()
                        input_field.send_keys(four_words)
                        print(f"   ✅ Entered four words: {four_words}")
                        inputs_filled += 1
                        break
                    except:
                        pass

                # Display name
                for selector in ["//input[contains(@placeholder, 'name')]", "//input[contains(@name, 'displayName')]", "//input[contains(@id, 'name')]"]:
                    try:
                        input_field = driver.find_element(By.XPATH, selector)
                        input_field.clear()
                        input_field.send_keys(display_name)
                        print(f"   ✅ Entered display name: {display_name}")
                        inputs_filled += 1
                        break
                    except:
                        pass

                # Device name
                for selector in ["//input[contains(@placeholder, 'device')]", "//input[contains(@name, 'deviceName')]", "//input[contains(@id, 'device')]"]:
                    try:
                        input_field = driver.find_element(By.XPATH, selector)
                        input_field.clear()
                        input_field.send_keys(device_name)
                        print(f"   ✅ Entered device name: {device_name}")
                        inputs_filled += 1
                        break
                    except:
                        pass

                driver.save_screenshot("test_4_form_filled.png")

                if inputs_filled > 0:
                    # Submit form
                    print("\n9️⃣ Submitting form...")

                    submit_selectors = [
                        "//button[contains(text(), 'Create Identity')]",
                        "//button[contains(text(), 'Create')]",
                        "//button[@type='submit']",
                        "//button[contains(text(), 'Continue')]"
                    ]

                    for selector in submit_selectors:
                        try:
                            submit_btn = driver.find_element(By.XPATH, selector)
                            if submit_btn.is_displayed() and submit_btn.is_enabled():
                                submit_btn.click()
                                print("   ✅ Form submitted!")
                                time.sleep(3)
                                break
                        except:
                            pass

                    driver.save_screenshot("test_5_after_submit.png")

                    # Check if we're now logged in
                    time.sleep(2)
                    try:
                        avatar = driver.find_element(By.CSS_SELECTOR, "[class*='Avatar']")
                        if avatar.is_displayed():
                            print("\n✅ Success! New user created and logged in")
                            results['steps'].append({
                                'step': 'Create and login new user',
                                'status': 'success'
                            })
                    except:
                        print("\n⚠️ Could not verify login status")
                else:
                    print("   ❌ Could not fill form fields")
        else:
            print("   ❌ Sign In button not found")

            # Debug: List all visible buttons
            print("\n   Debug - All visible buttons:")
            buttons = driver.find_elements(By.TAG_NAME, "button")
            for i, btn in enumerate(buttons[:10]):  # First 10 buttons
                if btn.is_displayed():
                    print(f"   {i+1}. '{btn.text}' | Enabled: {btn.is_enabled()}")

    except Exception as e:
        print(f"\n❌ Test failed: {e}")
        results['error'] = str(e)
        if driver:
            driver.save_screenshot("test_error.png")

    finally:
        if driver:
            # Final screenshot
            driver.save_screenshot("test_final.png")
            driver.quit()

        # Save results
        with open('test_auth_flow_results.json', 'w') as f:
            json.dump(results, f, indent=2)

        print("\n📊 Test Results Summary:")
        for step in results.get('steps', []):
            status_icon = "✅" if step['status'] == 'success' else "❌"
            print(f"   {status_icon} {step['step']}")

if __name__ == "__main__":
    test_full_auth_flow()