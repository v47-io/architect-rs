# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

 - 7 commits contributed to the release.
 - 7 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Updated version in Cargo manifest ([`455033c`](https://github.comgit//v47-io/architect-rs/commit/455033c48590cf85f671189fef87baf2517b82ee))
    - Added note about executable flag on Linux or macOS ([`7ffc9ab`](https://github.comgit//v47-io/architect-rs/commit/7ffc9ab6ebeb360fde5266e6e082aa711ce57501))
    - Unified path handling to preempt Windows UNC paths ([`48eb2c8`](https://github.comgit//v47-io/architect-rs/commit/48eb2c865afcf5aa618e71b9d66c0f6369fb6fa5))
    - Added installation docs and removed that part from README ([`8e5c63e`](https://github.comgit//v47-io/architect-rs/commit/8e5c63e8626ed168eac1e72f91d61a81ffc50609))
    - Now really fixed the introduction, including the README :D ([`6f25e35`](https://github.comgit//v47-io/architect-rs/commit/6f25e35a41cced51d8a4171062771656f250ac1e))
    - Fixed introduction page link ([`76d6439`](https://github.comgit//v47-io/architect-rs/commit/76d6439922ce9281f181bf3e1a0fc9f87215a380))
    - Added stub mdBooks docs and pipeline for publishing on gh-pages ([`5bfdb44`](https://github.comgit//v47-io/architect-rs/commit/5bfdb44d8100bb140630d9388cf34458904328a2))
</details>

## 0.9.0 (2021-10-29)

### New Features

- Cloning Git repositories as templates
- Configuration of template using JSON
- Rendering templates using Handlebars

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
