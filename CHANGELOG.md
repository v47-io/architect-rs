# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 1.1.0 (2021-12-02)

### Bug Fixes

 - <csr-id-8c657599dff5148d45c474b672069d66835642ab/> Corrected the behavior of conditional files
   Now evaluating all matching conditions if the file should be rendered or copied

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release over the course of 3 calendar days.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Updated version number for next release ([`d4a2206`](https://github.comgit//v47-io/architect-rs/commit/d4a2206f91f9991763418f542033b2d04e129266))
    - Corrected the behavior of conditional files ([`8c65759`](https://github.comgit//v47-io/architect-rs/commit/8c657599dff5148d45c474b672069d66835642ab))
</details>

## 1.0.0 (2021-11-28)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#12](https://github.comgit//v47-io/architect-rs/issues/12)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#12](https://github.comgit//v47-io/architect-rs/issues/12)**
    - Better release artifacts, and windows includes debug information ([`7b39f79`](https://github.comgit//v47-io/architect-rs/commit/7b39f79c8853bc8cc011133f406d966d81c3a4d3))
 * **Uncategorized**
    - Bump architect-rs v1.0.0 ([`1d23a04`](https://github.comgit//v47-io/architect-rs/commit/1d23a0454e47eb2052ac3639340ee6260c31fc10))
    - Updated version number for next release ([`90a92ce`](https://github.comgit//v47-io/architect-rs/commit/90a92ce594060a4e6833cd1c290c28e35a6fabc2))
</details>

## 0.15.0 (2021-11-28)

### New Features

 - <csr-id-45abfb590bd6743a05418e1a81671a92a5bf165c/> Implemented dry run feature
   This produces all the expected log output, without actually rendering or copying files to a target directory.
 - <csr-id-b59a0f1ab7169d829d035234745c5751f7ac925b/> Completely restyled terminal output and reorganized some output
 - <csr-id-a9a25ee8a24e25e059632f4bafdfc060a60c76e3/> Implemented custom theme for dialoguer
   Also pretty printing the context, and added crossterm to style verbose output

### Bug Fixes

 - <csr-id-3f3a26bca1afe7f322ebe8024cae9b47cdd3ce46/> Creating an empty Context instead of null Context
   `null` Context leads to issues further down the line because `build_file_context` expected a Context containing a `Value::Object(_)`, not `Value::Null`

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 11 commits contributed to the release over the course of 10 calendar days.
 - 10 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#6](https://github.comgit//v47-io/architect-rs/issues/6)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#6](https://github.comgit//v47-io/architect-rs/issues/6)**
    - Implemented dry run feature ([`45abfb5`](https://github.comgit//v47-io/architect-rs/commit/45abfb590bd6743a05418e1a81671a92a5bf165c))
    - Completely restyled terminal output and reorganized some output ([`b59a0f1`](https://github.comgit//v47-io/architect-rs/commit/b59a0f1ab7169d829d035234745c5751f7ac925b))
    - Implemented custom theme for dialoguer ([`a9a25ee`](https://github.comgit//v47-io/architect-rs/commit/a9a25ee8a24e25e059632f4bafdfc060a60c76e3))
 * **Uncategorized**
    - Bump architect-rs v0.15.0 ([`4561e69`](https://github.comgit//v47-io/architect-rs/commit/4561e6926fd3c3dd3cfb13308b49aa768bc5ccf3))
    - Updated some dependency versions and lockfile ([`0faa67b`](https://github.comgit//v47-io/architect-rs/commit/0faa67bfb24a51977f0996cba31e3f8bef57b3dd))
    - Added cache to test jobs ([`089b118`](https://github.comgit//v47-io/architect-rs/commit/089b1189d59480ddad2e0784e60e40072420c71d))
    - Updated version number for next release ([`adb128a`](https://github.comgit//v47-io/architect-rs/commit/adb128ad107529f7f7557fa2ed7fd97e36ac8387))
    - Merge pull request #14 from v47-io/feat/better-terminal-handling-#6 ([`9d00641`](https://github.comgit//v47-io/architect-rs/commit/9d00641a0b99dfc037fecf9ba4657ae9d9b45e4a))
    - Added expect test for dry-run feature ([`cb278b5`](https://github.comgit//v47-io/architect-rs/commit/cb278b54eb08879a7c7983e63dc1026ec0175809))
    - Implemented proper output for render conflicts ([`84b3d35`](https://github.comgit//v47-io/architect-rs/commit/84b3d35f22ae6db8d106cfc007dd3e5e37aff7ce))
    - Creating an empty Context instead of null Context ([`3f3a26b`](https://github.comgit//v47-io/architect-rs/commit/3f3a26bca1afe7f322ebe8024cae9b47cdd3ce46))
</details>

## 0.14.0 (2021-11-17)

### New Features

 - <csr-id-7c0e111c03fe03c7235025fba52cf56bd0a436e7/> Implemented custom formats for question values
   Provides the ability to define custom question formats using regular expressions. The default value is also validated using that regular expression.
   
   Moved the config module into its own directory.
   
   Added info about this to the documentation and fixed some small documentation issues.
   
   Updated the schema.json file

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#9](https://github.comgit//v47-io/architect-rs/issues/9)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#9](https://github.comgit//v47-io/architect-rs/issues/9)**
    - Implemented custom formats for question values ([`7c0e111`](https://github.comgit//v47-io/architect-rs/commit/7c0e111c03fe03c7235025fba52cf56bd0a436e7))
 * **Uncategorized**
    - Bump architect-rs v0.14.0 ([`03498c4`](https://github.comgit//v47-io/architect-rs/commit/03498c48cbf9ccac027964c90f6ae1b643c8e036))
    - Removed unnecessary dependency and updated version for next release ([`e041af9`](https://github.comgit//v47-io/architect-rs/commit/e041af9056411a9b5c9147342f3a04ac3ee01b13))
</details>

## 0.13.0 (2021-11-16)

### Documentation

 - <csr-id-afd37e546f2a988fec118b4041637ac64c1e563a/> Added note about embedded Git and local fallback
 - <csr-id-0b059c3ca3a73f05b8942521a361d9d5048adce8/> Added some information about local Git

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 12 commits contributed to the release over the course of 9 calendar days.
 - 9 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#4](https://github.comgit//v47-io/architect-rs/issues/4)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#4](https://github.comgit//v47-io/architect-rs/issues/4)**
    - Implemented fetching using embedded Git for portability ([`232408c`](https://github.comgit//v47-io/architect-rs/commit/232408c37feb928cbfb128f105d8ba84ea10165f))
 * **Uncategorized**
    - Bump architect-rs v0.13.0 ([`f239755`](https://github.comgit//v47-io/architect-rs/commit/f239755337c2bffcb1340b86449c2ca0b302eab5))
    - Release architect-rs v0.13.0 ([`91640f2`](https://github.comgit//v47-io/architect-rs/commit/91640f21b59875336bc4e121751881a7f9bacf40))
    - Didn't need to update the version before release ([`d198f0b`](https://github.comgit//v47-io/architect-rs/commit/d198f0b49c2685d555583244ff8b16df6fb53a74))
    - Added note about embedded Git and local fallback ([`afd37e5`](https://github.comgit//v47-io/architect-rs/commit/afd37e546f2a988fec118b4041637ac64c1e563a))
    - Updated version number for next release ([`1953e7a`](https://github.comgit//v47-io/architect-rs/commit/1953e7a46e5bb83a5d296e80dd0294238b285346))
    - Added some input processing to embedded git ([`09d013e`](https://github.comgit//v47-io/architect-rs/commit/09d013e3a6a299cfd8bb9785c47bd479e2376b47))
    - Updated edition to 2021 ([`847ea5c`](https://github.comgit//v47-io/architect-rs/commit/847ea5c0ea3f4519de17f959eb5f72383beaac28))
    - Merge pull request #13 from v47-io/feature/handle-git-internally-without-bin-dependency ([`bb1d9c0`](https://github.comgit//v47-io/architect-rs/commit/bb1d9c0422c621accaf84078598aef4b903c03bd))
    - Added some basic tests to the fetch module ([`ca7bfcd`](https://github.comgit//v47-io/architect-rs/commit/ca7bfcd0e81f3f223e55d1fba33c10a3dd812733))
    - Added some information about local Git ([`0b059c3`](https://github.comgit//v47-io/architect-rs/commit/0b059c3ca3a73f05b8942521a361d9d5048adce8))
    - Updated version number for next release ([`88b45a8`](https://github.comgit//v47-io/architect-rs/commit/88b45a80f22be8b814c1b5f34ab65334b082a4f0))
</details>

## 0.12.0 (2021-11-06)

### Documentation

 - <csr-id-288280cfbaf80ab2d647d189f8c57acd556a7059/> Added information about default values for questions

### New Features

 - <csr-id-addcd66a3c4d546d124632d47b946656debda8fe/> Default values for questions
   Works for text, identifiers, selections, multi-selections, and options.

### Bug Fixes

 - <csr-id-6b3ef343d2a569ab47b1788eb26ec6055e91484d/> Handling of questions asking for Identifiers, now accepting dot-concatenated identifiers

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 12 commits contributed to the release over the course of 1 calendar day.
 - 10 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#5](https://github.comgit//v47-io/architect-rs/issues/5)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#5](https://github.comgit//v47-io/architect-rs/issues/5)**
    - Default values for questions ([`addcd66`](https://github.comgit//v47-io/architect-rs/commit/addcd66a3c4d546d124632d47b946656debda8fe))
 * **Uncategorized**
    - Release architect-rs v0.12.0 ([`d2c26f6`](https://github.comgit//v47-io/architect-rs/commit/d2c26f69f9806d41c08a1fee5838827de150b590))
    - Expanded test-architect-select as bit to also test option ([`6ee5548`](https://github.comgit//v47-io/architect-rs/commit/6ee55484546d97c0c7f135a5d9e42ec2df220f29))
    - Added information about default values for questions ([`288280c`](https://github.comgit//v47-io/architect-rs/commit/288280cfbaf80ab2d647d189f8c57acd556a7059))
    - Added some more detail to the sample config ([`7a23bf2`](https://github.comgit//v47-io/architect-rs/commit/7a23bf28d9af249c99f5ff34cf58f89f3c038cd0))
    - Merge pull request #10 from v47-io/feature/default-value-for-questions ([`f9789af`](https://github.comgit//v47-io/architect-rs/commit/f9789affaab58be137eaf69a32fbad4cd2b1eb3f))
    - Handling of questions asking for Identifiers, now accepting dot-concatenated identifiers ([`6b3ef34`](https://github.comgit//v47-io/architect-rs/commit/6b3ef343d2a569ab47b1788eb26ec6055e91484d))
    - Fixed escape in expect for selection test ([`d083682`](https://github.comgit//v47-io/architect-rs/commit/d083682f70d23053e166d4f72d3c8dc6c8fcfd12))
    - Fixed multi-selection ([`292d607`](https://github.comgit//v47-io/architect-rs/commit/292d60734b421998875d7fbc84cfc15c9d8d598e))
    - Added another expect file to test the selection input ([`78c5b1e`](https://github.comgit//v47-io/architect-rs/commit/78c5b1e231335056c968db699e628147c3c6916d))
    - Added tests for the new default value config ([`b2b4797`](https://github.comgit//v47-io/architect-rs/commit/b2b4797198f3f6e2a49a6f28bbfb29aebe6107b5))
    - Added docs shortcut links to README ([`4f348fd`](https://github.comgit//v47-io/architect-rs/commit/4f348fdebd642bbfe15c16a187c94a3f95b3578d))
</details>

## 0.11.0 (2021-11-05)

### New Features

 - <csr-id-b0e832662ce9fdad1b23f4fe4353690a65cd954d/> WIP Implemented using subdirectory of template repo as template
   This implements repositories of templates, each in their own directory. Currently working on rewriting the Git history to fit subdirectory instead of entire repository.

### Bug Fixes

 - <csr-id-36312af87ebf7f29d95bc2213ed9ef588c68e6e7/> Expanded filter to check for sub template dirs and exclude them
 - <csr-id-9c513cce1cfb067975d755e53b65c15f432e2f67/> Updated TLDR

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 16 commits contributed to the release over the course of 5 calendar days.
 - 13 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#3](https://github.comgit//v47-io/architect-rs/issues/3)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#3](https://github.comgit//v47-io/architect-rs/issues/3)**
    - Allow- and deny-list for template generation in configuration ([`cf1e019`](https://github.comgit//v47-io/architect-rs/commit/cf1e01940b753c4f6454b116680716b39a83c7ea))
 * **Uncategorized**
    - Release architect-rs v0.11.0 ([`2230a99`](https://github.comgit//v47-io/architect-rs/commit/2230a992797d4e1241218d023b5d4922c57b0ffc))
    - Bump version ([`2cc1c85`](https://github.comgit//v47-io/architect-rs/commit/2cc1c854dc7222195fe5d7ad10f3ab96070014c7))
    - Fixed formatting ([`b717fc3`](https://github.comgit//v47-io/architect-rs/commit/b717fc3f63f41cc43408b6f93c4db21740e27c43))
    - Added log statement ([`1f4936d`](https://github.comgit//v47-io/architect-rs/commit/1f4936d21b0653f6b25fff2a258e8fda47af2027))
    - Expanded filter to check for sub template dirs and exclude them ([`36312af`](https://github.comgit//v47-io/architect-rs/commit/36312af87ebf7f29d95bc2213ed9ef588c68e6e7))
    - Merge pull request #8 from v47-io/feat/#7-multi-template-repos ([`16234e6`](https://github.comgit//v47-io/architect-rs/commit/16234e6bdee66da9ae473471b8e8f06c4d449141))
    - Fixed expect file for test-binary job ([`fd0085a`](https://github.comgit//v47-io/architect-rs/commit/fd0085a7e1ad52331c3e7562555d447b9b44fc92))
    - Added test for `find_template_dir` ([`fec7a70`](https://github.comgit//v47-io/architect-rs/commit/fec7a70f9b2d8eeb9514693c05c2e2b695474a2f))
    - Got rid of Git tree rewriting, that's a task for another day ([`52c4f8c`](https://github.comgit//v47-io/architect-rs/commit/52c4f8cc9c44078cdc951dc9cbe7e8e9131bedf2))
    - Executing tests, and doc building on all branches ([`77f461a`](https://github.comgit//v47-io/architect-rs/commit/77f461afb39b18e9c32d7102162dfba2c8d5f307))
    - WIP Implemented using subdirectory of template repo as template ([`b0e8326`](https://github.comgit//v47-io/architect-rs/commit/b0e832662ce9fdad1b23f4fe4353690a65cd954d))
    - Added a sample config to the README ([`6bc33ec`](https://github.comgit//v47-io/architect-rs/commit/6bc33ecebf7deb3fca64cf1c8fc8cff88d0fed79))
    - Added a note to README about project and open issues ([`de29a70`](https://github.comgit//v47-io/architect-rs/commit/de29a70c0039c499c4c48e0ceab48f46ce6a85f7))
    - Update issue templates ([`4144fa9`](https://github.comgit//v47-io/architect-rs/commit/4144fa92d8e8cb6a6594985c28e87290b8f1a543))
    - Updated TLDR ([`9c513cc`](https://github.comgit//v47-io/architect-rs/commit/9c513cce1cfb067975d755e53b65c15f432e2f67))
</details>

## 0.10.0 (2021-10-30)

### Documentation

 - <csr-id-1a3a15e212cbda563ec746d6e54514d41f1b551d/> Added documentation about expert topics
 - <csr-id-cf18c373fb73dedade1c428e54100de16253b314/> Added documentation for rendering and removed those parts from README
 - <csr-id-186d41dc11bf9d13a4c8c82e73313d2f8a67b860/> Fixed a link to the configuration page
 - <csr-id-f984255a43afab66400bde1f94dda4a2f03af32b/> Added bunch more docs for the configuration file and the context
 - <csr-id-ce7946c3f27cf49dd1c345a83c7500c91a38cd97/> Wrote documentation for the templates, fetching, and structure
 - <csr-id-643bcd04ebf8a1eb3830ba13d1d61d14b7ca1435/> Fixed special characters
 - <csr-id-dd2a1367a6bcca70e5b14aef507849448dbf8cc9/> Added CLI documentation and removed usage from README

### Bug Fixes

 - <csr-id-effd69fd13c1e2be1513b3b01027d060aeeca104/> Added more badges to README 🎉
 - <csr-id-d8e0e6e15c5001de16732db967d8f510b6d9c94e/> Even better README
 - <csr-id-5a866d109431a152e2013daf53acd51cf1064250/> Better README
 - <csr-id-2c3ac6b3a29d9a9132d8edae7b4e67e3c21ad46f/> Better README
 - <csr-id-13a63432d1ec5f61a58015ed45e0cc7152ebdcca/> Fixed handling of glob expressions according to docs

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 13 commits contributed to the release.
 - 12 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release architect-rs v0.10.0 ([`b1924aa`](https://github.comgit//v47-io/architect-rs/commit/b1924aab7b6efa999f539ac5eef1dffcc87dfe36))
    - Added more badges to README 🎉 ([`effd69f`](https://github.comgit//v47-io/architect-rs/commit/effd69fd13c1e2be1513b3b01027d060aeeca104))
    - Even better README ([`d8e0e6e`](https://github.comgit//v47-io/architect-rs/commit/d8e0e6e15c5001de16732db967d8f510b6d9c94e))
    - Added documentation about expert topics ([`1a3a15e`](https://github.comgit//v47-io/architect-rs/commit/1a3a15e212cbda563ec746d6e54514d41f1b551d))
    - Better README ([`5a866d1`](https://github.comgit//v47-io/architect-rs/commit/5a866d109431a152e2013daf53acd51cf1064250))
    - Better README ([`2c3ac6b`](https://github.comgit//v47-io/architect-rs/commit/2c3ac6b3a29d9a9132d8edae7b4e67e3c21ad46f))
    - Added documentation for rendering and removed those parts from README ([`cf18c37`](https://github.comgit//v47-io/architect-rs/commit/cf18c373fb73dedade1c428e54100de16253b314))
    - Fixed handling of glob expressions according to docs ([`13a6343`](https://github.comgit//v47-io/architect-rs/commit/13a63432d1ec5f61a58015ed45e0cc7152ebdcca))
    - Fixed a link to the configuration page ([`186d41d`](https://github.comgit//v47-io/architect-rs/commit/186d41dc11bf9d13a4c8c82e73313d2f8a67b860))
    - Added bunch more docs for the configuration file and the context ([`f984255`](https://github.comgit//v47-io/architect-rs/commit/f984255a43afab66400bde1f94dda4a2f03af32b))
    - Wrote documentation for the templates, fetching, and structure ([`ce7946c`](https://github.comgit//v47-io/architect-rs/commit/ce7946c3f27cf49dd1c345a83c7500c91a38cd97))
    - Fixed special characters ([`643bcd0`](https://github.comgit//v47-io/architect-rs/commit/643bcd04ebf8a1eb3830ba13d1d61d14b7ca1435))
    - Added CLI documentation and removed usage from README ([`dd2a136`](https://github.comgit//v47-io/architect-rs/commit/dd2a1367a6bcca70e5b14aef507849448dbf8cc9))
</details>

## 0.9.1 (2021-10-30)

### Documentation

 - <csr-id-7ffc9ab6ebeb360fde5266e6e082aa711ce57501/> Added note about executable flag on Linux or macOS
 - <csr-id-8e5c63e8626ed168eac1e72f91d61a81ffc50609/> Added installation docs and removed that part from README
 - <csr-id-6f25e35a41cced51d8a4171062771656f250ac1e/> Now really fixed the introduction, including the README :D
 - <csr-id-76d6439922ce9281f181bf3e1a0fc9f87215a380/> Fixed introduction page link
 - <csr-id-5bfdb44d8100bb140630d9388cf34458904328a2/> Added stub mdBooks docs and pipeline for publishing on gh-pages

### Bug Fixes

 - <csr-id-48eb2c865afcf5aa618e71b9d66c0f6369fb6fa5/> Unified path handling to preempt Windows UNC paths

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 8 commits contributed to the release.
 - 7 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release architect-rs v0.9.1 ([`b8267be`](https://github.comgit//v47-io/architect-rs/commit/b8267bee88846d5da53ac22069dbd06cbc181d6a))
    - Updated version in Cargo manifest ([`455033c`](https://github.comgit//v47-io/architect-rs/commit/455033c48590cf85f671189fef87baf2517b82ee))
    - Added note about executable flag on Linux or macOS ([`7ffc9ab`](https://github.comgit//v47-io/architect-rs/commit/7ffc9ab6ebeb360fde5266e6e082aa711ce57501))
    - Unified path handling to preempt Windows UNC paths ([`48eb2c8`](https://github.comgit//v47-io/architect-rs/commit/48eb2c865afcf5aa618e71b9d66c0f6369fb6fa5))
    - Added installation docs and removed that part from README ([`8e5c63e`](https://github.comgit//v47-io/architect-rs/commit/8e5c63e8626ed168eac1e72f91d61a81ffc50609))
    - Now really fixed the introduction, including the README :D ([`6f25e35`](https://github.comgit//v47-io/architect-rs/commit/6f25e35a41cced51d8a4171062771656f250ac1e))
    - Fixed introduction page link ([`76d6439`](https://github.comgit//v47-io/architect-rs/commit/76d6439922ce9281f181bf3e1a0fc9f87215a380))
    - Added stub mdBooks docs and pipeline for publishing on gh-pages ([`5bfdb44`](https://github.comgit//v47-io/architect-rs/commit/5bfdb44d8100bb140630d9388cf34458904328a2))
</details>

## 0.9.0 (2021-10-29)

### Bug Fixes

 - <csr-id-9df8d44d2a4a67bf0bf9a9b594efded44b395601/> Properly handling the result in `is_not_excluded`
 - <csr-id-b92417644a026953d81da8a147f1a28d0d4a8551/> Fixed split by wrong char
   Exactly the opposite actually, split by `/` and join by `.` Doh!!! 🤦

### New Features

- Cloning Git repositories as templates
- Configuration of template using JSON
- Rendering templates using Handlebars
 - <csr-id-60dc3211bab34df82bf851691cfa41712b02be46/> Implemented configurable treatment of actual Handlebars template files
   This is done so users can have actual Handlebars template files in their repos without them being rendered.
 - <csr-id-7f1f5af5d3cb769c8437d9677d384e4f00c381e0/> Added template file metadata to rendering context

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release architect-rs v0.9.0 ([`0ba191b`](https://github.comgit//v47-io/architect-rs/commit/0ba191ba88dbbed59333b9b32f197b5a16738d9b))
    - Prepared for release ([`3c3fecf`](https://github.comgit//v47-io/architect-rs/commit/3c3fecf99e17ac9665c679743d780f9678005912))
    - 🤦‍ ([`30f46be`](https://github.comgit//v47-io/architect-rs/commit/30f46be42803dc56b790218b0e40daef9eb1eaa2))
    - Fixed closing paren ([`129da75`](https://github.comgit//v47-io/architect-rs/commit/129da75e3cdb39381a15ad82cf1d949c382d9dde))
    - Fixed version and tag handling ([`477c2fc`](https://github.comgit//v47-io/architect-rs/commit/477c2fce4544f2da9dab77aecd6c68935b7a6927))
    - Properly preparing release assets before upload ([`48b1f90`](https://github.comgit//v47-io/architect-rs/commit/48b1f905c30d7eb744e9223b6ea0f7e6144bfba8))
    - Creating draft release and uploading artifacts ([`f7c13f6`](https://github.comgit//v47-io/architect-rs/commit/f7c13f6898244762e412caa22c94b328ee9d1908))
    - Preparing for automated release on tag commit ([`7094782`](https://github.comgit//v47-io/architect-rs/commit/709478268a9ae0b63f3c6178f4a24c36a177ef0b))
    - Updated readme and quality pipeline name ([`f48c616`](https://github.comgit//v47-io/architect-rs/commit/f48c616b03ffa380f425329da63a3a69f89393c0))
    - Proper artifact names ([`5ec01da`](https://github.comgit//v47-io/architect-rs/commit/5ec01dac25f5fdb354fdf41df4c3546f35fd56e3))
    - Fixed artifact names ([`2050b9f`](https://github.comgit//v47-io/architect-rs/commit/2050b9f271874e085b778a094f59310cc11577e0))
    - Added publish pipeline ([`b15c36d`](https://github.comgit//v47-io/architect-rs/commit/b15c36db617f74dbec585dfa3f575f684575dc75))
    - Added extra assertion to check if https URL is remote spec ([`34eb02a`](https://github.comgit//v47-io/architect-rs/commit/34eb02a5b0d8756b2795775b6ef002e2122de4c1))
    - Added tests for fetching repos ([`e822212`](https://github.comgit//v47-io/architect-rs/commit/e82221205a8a37e9413be6dbb0a0906f3040ec95))
    - Verifying the rendered file list ([`24ff1e4`](https://github.comgit//v47-io/architect-rs/commit/24ff1e43810596d7c71d68f30914b0e5874d21ae))
    - Fixed clippy complaints and enabled clippy step in lint job ([`4c26113`](https://github.comgit//v47-io/architect-rs/commit/4c26113e2f6cfcaba9b4b4665fe461f8dab228c4))
    - Copy-paste strikes again, fixed rust toolchain for test-binary job ([`b51c19d`](https://github.comgit//v47-io/architect-rs/commit/b51c19df2d64761620152c2c7d281c4777af4033))
    - Fixed execution of apt commands ([`29ce2e7`](https://github.comgit//v47-io/architect-rs/commit/29ce2e7e43f28e8e708f6610634013a535383622))
    - Added additional job to GitHub actions to execute the final architect binary, just to make sure it works ([`314c079`](https://github.comgit//v47-io/architect-rs/commit/314c07997696be12512eafd17425f42f8a630f39))
    - Added additional assertion to check if the explicit Handlebars template file is handle appropriately ([`9003c09`](https://github.comgit//v47-io/architect-rs/commit/9003c09c1216e1a344445f4ec5936da002a3c06d))
    - Added test for `render` 🎉 ([`73c06cf`](https://github.comgit//v47-io/architect-rs/commit/73c06cfa285e4fb2ad62acd33ad2d136292fa4cb))
    - Added test for `build_file_context` ([`8c28a83`](https://github.comgit//v47-io/architect-rs/commit/8c28a83a3b7de6dd7848c5d061f6d251ed9022e9))
    - Properly handling the result in `is_not_excluded` ([`9df8d44`](https://github.comgit//v47-io/architect-rs/commit/9df8d44d2a4a67bf0bf9a9b594efded44b395601))
    - Process exclusions of hidden files and dirs only based on full paths, not just singular file names ([`a002479`](https://github.comgit//v47-io/architect-rs/commit/a002479528cf37a00fad0154069992b324afa28e))
    - Small teensy weensy little changes in tests, nothing to write home about ([`4cfe6a6`](https://github.comgit//v47-io/architect-rs/commit/4cfe6a61427f165ef5bd329ac9d4defacb15f4fb))
    - Added incomplete test for `build_render_specs` in render module ([`94c6f4c`](https://github.comgit//v47-io/architect-rs/commit/94c6f4ca70645b4492b20eccf495c1505005282a))
    - Fixed split by wrong char ([`b924176`](https://github.comgit//v47-io/architect-rs/commit/b92417644a026953d81da8a147f1a28d0d4a8551))
    - Added template file metadata to rendering context ([`7f1f5af`](https://github.comgit//v47-io/architect-rs/commit/7f1f5af5d3cb769c8437d9677d384e4f00c381e0))
    - Added test template files for render module tests ([`4f2e61b`](https://github.comgit//v47-io/architect-rs/commit/4f2e61b5aa1c2b41990874d5e751ef8261306e57))
    - Implemented configurable treatment of actual Handlebars template files ([`60dc321`](https://github.comgit//v47-io/architect-rs/commit/60dc3211bab34df82bf851691cfa41712b02be46))
    - Fixed test ([`dde08b1`](https://github.comgit//v47-io/architect-rs/commit/dde08b14a10af7c20abda668845d0bd1115935f2))
    - Added a missing assertion for `include_dir_entry` ([`4c817c7`](https://github.comgit//v47-io/architect-rs/commit/4c817c7d13d6910152b3aad4be39caaed4f35fea))
    - Added tests for `include_dir_entry` and `is_hbs_template` ([`5805885`](https://github.comgit//v47-io/architect-rs/commit/58058856130d429a9c78adbb57e29867c33e1153))
    - Added tests for dirs module ([`22186c5`](https://github.comgit//v47-io/architect-rs/commit/22186c5c6f454b2ba9b2b7bafaa1f1d6a6516ee7))
    - Implemented test for BufReader ([`b06fead`](https://github.comgit//v47-io/architect-rs/commit/b06fead87a6eaefad6423bc5d47764ca88a676a9))
    - Major changes ([`473f66e`](https://github.comgit//v47-io/architect-rs/commit/473f66e1ac3b01127c9e0edd34f93928ab7765a3))
    - Fixed line endings, changed CRLF to LF ([`c722fb0`](https://github.comgit//v47-io/architect-rs/commit/c722fb09b912a08f7eac8933a3d14fa359f64b2a))
    - Added a bunch more tests, especially in the render module ([`5d441ed`](https://github.comgit//v47-io/architect-rs/commit/5d441edd9cc121f2244c48fe272d3b537b55e450))
    - Refactored args matching into separate module, added a few todos ([`24017aa`](https://github.comgit//v47-io/architect-rs/commit/24017aa3beafb277e1b574ae172483e182be3ab3))
    - Added some tests to the render module and a small fix in the spec module ([`c542152`](https://github.comgit//v47-io/architect-rs/commit/c542152ea3661944e8f9aa535891fcb9d8393946))
    - Removed useless use ([`56ef52f`](https://github.comgit//v47-io/architect-rs/commit/56ef52f4a6d815d9207e309f94dd6563ac173ca8))
    - Made template name and version both optional in config file ([`6050b8e`](https://github.comgit//v47-io/architect-rs/commit/6050b8e84747c326cd968068ef6ffd78219d01e3))
    - Renamed "lenient" to the more appropriate "ignore-checks" and removed short flag ([`6c76dfc`](https://github.comgit//v47-io/architect-rs/commit/6c76dfcdba7870fbf97620e935ef840df4e92d9d))
    - Updated readme ([`0910f00`](https://github.comgit//v47-io/architect-rs/commit/0910f00f8779b058f1dca495fe5d6a798de03e3b))
    - Redesigned conditional directory and file generation ([`3f508ff`](https://github.comgit//v47-io/architect-rs/commit/3f508ff9aed018f6cacdab421c7ac4cf70be46f6))
    - Small readme fix ([`7fbee17`](https://github.comgit//v47-io/architect-rs/commit/7fbee179726a58acb4033150a6f0f7e2bed386ac))
    - Created proper package helper to convert package names to paths ([`47ff3b8`](https://github.comgit//v47-io/architect-rs/commit/47ff3b88173b5a5aaa8bb83ab8510c1e79d16fbe))
    - Small formatting fix in README ([`ba880eb`](https://github.comgit//v47-io/architect-rs/commit/ba880ebea561e976fac9c433d4898fa2b53f405e))
    - Added codecov badge ([`18459ec`](https://github.comgit//v47-io/architect-rs/commit/18459ec6166a9959437075df5ba027305815c0a6))
    - Trying the cargo-binutils route ([`8e447fb`](https://github.comgit//v47-io/architect-rs/commit/8e447fb4ded91ed4ec41bcca75cff9992b43c534))
    - Using absolute path to installed tools ([`ee6040f`](https://github.comgit//v47-io/architect-rs/commit/ee6040fdd35424fc9e192a17b6f79d0426d19f0f))
    - Need to use nightly channel ([`063fa6b`](https://github.comgit//v47-io/architect-rs/commit/063fa6b14061941483a4b59604136a6c239c0590))
    - Doing the coverage myself ([`3c2671f`](https://github.comgit//v47-io/architect-rs/commit/3c2671f559b375069ca1da7df7ef7fbe1c5517c1))
    - Added build badge to readme, and coverage to workflow ([`c80e02c`](https://github.comgit//v47-io/architect-rs/commit/c80e02cd821f39bf359dbb58bf491105a955e306))
    - Increased test-ability by explicitly specifying args ([`6437739`](https://github.comgit//v47-io/architect-rs/commit/6437739ae9b1fcf06c109e87baf7a4f8fafb7744))
    - Added the unix counterparts for the windows based tests ([`0ec2591`](https://github.comgit//v47-io/architect-rs/commit/0ec2591a40ad7266d92e9fafe8e44ab97736c38d))
    - Added tests for parts of context creation ([`922f5f0`](https://github.comgit//v47-io/architect-rs/commit/922f5f0e39c0cceb90301761daf4c1423ad7fb70))
    - Need to remove the handlebars extension from template files ([`0f99639`](https://github.comgit//v47-io/architect-rs/commit/0f9963937a41cee297085e6e900fe7b2384fa261))
    - Fixed tests ([`6e0b22c`](https://github.comgit//v47-io/architect-rs/commit/6e0b22cb93ec67c5f7b4fc2d89aed28807e7cd27))
    - Added license and contributing info ([`76f28c3`](https://github.comgit//v47-io/architect-rs/commit/76f28c327a85a244bcc1875ac7ef1008b71a5a23))
    - Small fix ([`b347bc6`](https://github.comgit//v47-io/architect-rs/commit/b347bc6dd9b3012ab0da5783a67446e95a134a72))
    - Finalized desired functionality and added README ([`f80b02f`](https://github.comgit//v47-io/architect-rs/commit/f80b02f0e1ec2d2371cd1e605425dcad7bc4a5c3))
    - Finished implementation, just need to add more tests ([`0334bac`](https://github.comgit//v47-io/architect-rs/commit/0334bac3585486cc9f1ee89c1b3d90172ae319e9))
    - Overhauled config loading and added tests ([`bfad64a`](https://github.comgit//v47-io/architect-rs/commit/bfad64ad82e94dc8c5ebbae9a0ff3dfb87b40afd))
    - So, that was a disaster... ([`f772fb6`](https://github.comgit//v47-io/architect-rs/commit/f772fb6ca1be7c5ff85f0f06e038222c7c663e5f))
    - Fixed some clippy complaints but disabled it for now in the workflows ([`8fd89de`](https://github.comgit//v47-io/architect-rs/commit/8fd89dea6eb8b379cf978ab860a44e473b770f11))
    - Fixed template spec tests ([`af87c20`](https://github.comgit//v47-io/architect-rs/commit/af87c20fd3fc793155f6ea828a24b97e8645e8c8))
    - Added Github actions for building ([`f39878b`](https://github.comgit//v47-io/architect-rs/commit/f39878b46bd7c927bf68271ff2122edcce3920b0))
    - Added some tests to spec and utils modules ([`f724a5c`](https://github.comgit//v47-io/architect-rs/commit/f724a5c0b4670eb869e637688c7985aab18fd787))
    - Implemented nesting values in the context, also checking for duplicates and incompatible names ([`83e5895`](https://github.comgit//v47-io/architect-rs/commit/83e589560c2345f29ed5b23704fb6ae7941db572))
    - Implemented the parsing of the template configuration file ([`069e218`](https://github.comgit//v47-io/architect-rs/commit/069e218dc2b0add9bc877bc30e71c5a9f4947ddd))
    - Initial commit ([`0999f19`](https://github.comgit//v47-io/architect-rs/commit/0999f19bcf3f68154e6086295e6de7aae1f735b6))
</details>

