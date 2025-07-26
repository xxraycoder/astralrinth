import { ref } from 'vue'
import { getVersion } from '@tauri-apps/api/app'
import { initUpdateLauncher, getOS } from '@/helpers/utils.js'

export const allowState = ref(false)
export const installState = ref(false)
export const updateState = ref(false)

const currentOS = ref('')
const releaseLink = `https://git.astralium.su/api/v1/repos/didirus/AstralRinth/releases/latest`
const failedFetch = [`Failed to fetch remote releases:`, `Failed to fetch remote commits:`]

const osList = ['macos', 'windows', 'linux']
const macExtensionList = ['.dmg', '.pkg']
const windowsExtensionList = ['.exe', '.msi']

const blacklistPrefixes = [
  `dev`,
  `nightly`,
  `dirty`,
  `dirty-dev`,
  `dirty-nightly`,
  `dirty_dev`,
  `dirty_nightly`,
] // This is blacklisted builds for download. For example, file.startsWith('dev') is not allowed.

export async function getRemote(isDownloadState) {
  var releaseData = null;
  var result = false;
  try {
    const response = await fetch(releaseLink);
    if (!response.ok) {
      throw new Error(response.status);
    }
    const remoteData = await response.json();
    currentOS.value = await getOS();
    const remoteLatestReleaseTag = remoteData.tag_name;
    releaseData = document.getElementById('releaseData');
    const remoteVersion = releaseData ? (releaseData.textContent = remoteLatestReleaseTag) : remoteLatestReleaseTag;

    if (osList.includes(currentOS.value.toLowerCase())) {
      const localVersion = await getVersion();
      const isUpdateAvailable = !remoteVersion.includes(localVersion);

      updateState.value = isUpdateAvailable;
      allowState.value = isUpdateAvailable;
    } else {
      updateState.value = false;
      allowState.value = false;
    }
    if (isDownloadState) {
      installState.value = true;
      const builds = remoteData.assets;
      const fileName = getInstaller(getExtension(), builds);
      result = fileName ? await initUpdateLauncher(fileName[1], fileName[0], currentOS.value, true) : false;
      installState.value = false;
    }

    console.log('Update available state is', updateState.value);
    console.log('Remote version is', remoteVersion);
    console.log('Local version is', await getVersion());
    console.log('Operating System is', currentOS.value);
    return result;
  } catch (error) {
    console.error(failedFetch[0], error);
    if (!releaseData) {
      const errorData = document.getElementById('releaseData');
      if (errorData) {
        errorData.textContent = `${error.message}`;
      }
      updateState.value = false;
      allowState.value = false;
      installState.value = false;
    }
  }
}

function getInstaller(osExtension, builds) {
  console.log(osExtension, builds)
  for (const build of builds) {
    if (blacklistPrefixes.some(prefix => build.name.startsWith(prefix))) {
      continue;
    }
    if (osExtension.some(ext => build.name.endsWith(ext))) {
      console.log(build.name, build.browser_download_url);
      return [build.name, build.browser_download_url];
    }
  }
  return null;
}

function getExtension() {
  return osList.find(osName => osName === currentOS.value.toLowerCase())?.endsWith('macos')
    ? macExtensionList
    : windowsExtensionList;
}
