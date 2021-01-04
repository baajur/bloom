/* eslint-disable class-methods-use-this */
/* eslint-disable max-len */
import APIClient from '@/api/client';
import {
  DeleteMyAccountInput,
  DisableTwoFaInput,
  EnableTwoFaInput,
  Group, SignedStorageUploadUrl, StatusPage, UpdateGroupProfileInput, UpdateMyProfileInput, User,
} from '@/api/graphql/model';
import { AppState, Mutation } from '@/app/store';
import { Store } from 'vuex';

export type StorageSignedUploadUrlInput = {
  size: number;
}

export class KernelService {
  private apiClient: APIClient;
  private store: Store<AppState>;

  constructor(apiClient: APIClient, store: Store<AppState>) {
    this.apiClient = apiClient;
    this.store = store;
  }

  async storageSignedUploadUrl(input: StorageSignedUploadUrlInput): Promise<SignedStorageUploadUrl> {
    const query = `
      query($fileSize: Int64!) {
        signedStorageUploadUrl(fileSize: $fileSize) {
          url
          size
          tmpKey
        }
      }
    `;
    const variables = { fileSize: input.size };

    const res: { signedStorageUploadUrl: SignedStorageUploadUrl } = await this.apiClient.query(query, variables);
    return res.signedStorageUploadUrl;
  }

  validateAvatar(file: File) {
    if (file.type !== 'image/jpeg' && file.type !== 'image/png') {
      throw new Error('Image format must be png, jpg or jpeg');
    }

    // 2 MB
    if (file.size > 2000000) {
      throw new Error('File size must be less or equal to 2MB');
    }
  }

  async updateMyAvatar(file: File): Promise<string> {
    this.validateAvatar(file);

    const query = `
      mutation($input: UpdateMyProfileInput!) {
        updateMyProfile(input: $input) {
          id
          avatarUrl
        }
      }
    `;
    const input: UpdateMyProfileInput = {};
    const variables = { input };
    const operations = { query, variables };
    const map = {
      0: ['variables.input.avatar'],
    };

    const formData = new FormData();
    formData.append('operations', JSON.stringify(operations));
    formData.append('map', JSON.stringify(map));
    formData.append('0', file);

    const res: { updateMyProfile: User } = await this.apiClient.upload(formData);
    this.store.commit(Mutation.UPDATE_MY_PROFILE, res.updateMyProfile);
    return res.updateMyProfile.avatarUrl;
  }

  async updateGroupAvatar(groupId: string, file: File): Promise<string> {
    this.validateAvatar(file);

    const query = `
      mutation($input: UpdateGroupProfileInput!) {
        updateGroupProfile(input: $input) {
          id
          avatarUrl
        }
      }
    `;
    const input: UpdateGroupProfileInput = {
      id: groupId,
    };
    const variables = { input };
    const operations = { query, variables };
    const map = {
      0: ['variables.input.avatar'],
    };

    const formData = new FormData();
    formData.append('operations', JSON.stringify(operations));
    formData.append('map', JSON.stringify(map));
    formData.append('0', file);

    const res: { updateGroupProfile: Group } = await this.apiClient.upload(formData);
    return res.updateGroupProfile.avatarUrl;
  }

  async deleteMyAccount(input: DeleteMyAccountInput): Promise<void> {
    const query = `
      mutation($input: DeleteMyAccountInput!) {
        deleteMyAccount(input: $input)
      }
    `;
    const variables = { input };

    await this.apiClient.query(query, variables);
    this.store.commit(Mutation.SIGN_OUT);
    window.location.href = '/';
  }

  async fetchStatusPage(projectFullPath: string): Promise<StatusPage> {
    const query = `
      query($projectFullPath: String!) {
        statusPage(projectFullPath: $projectFullPath) {
          name
          avatarUrl
          twitterUrl
          facebookUrl
          publicEmail
          instagramUrl
          whatsappNumber
          mastodonUrl
          homepageUrl

          monitors {
            name
            status
          }
        }
      }
    `;
    const variables = { projectFullPath };

    const res: { statusPage: StatusPage } = await this.apiClient.query(query, variables);
    return res.statusPage;
  }

  async setupTwoFA(): Promise<string> {
    const query = `
      mutation {
        setupTwoFA
      }
    `;
    const variables = { };

    const res: { setupTwoFA: string } = await this.apiClient.query(query, variables);
    return res.setupTwoFA;
  }

  async enableTwoFA(input: EnableTwoFaInput): Promise<void> {
    const query = `
      mutation($input: EnableTwoFAInput!) {
        enableTwoFA(input: $input)
      }
    `;
    const variables = { input };

    await this.apiClient.query(query, variables);
  }

  async disableTwoFA(input: DisableTwoFaInput): Promise<void> {
    const query = `
      mutation($input: DisableTwoFAInput!) {
        disableTwoFA(input: $input)
      }
    `;
    const variables = { input };

    await this.apiClient.query(query, variables);
  }
}

export const KernelServiceInjector = {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  install(Vue: any, service: KernelService) {
    Vue.prototype.$kernelService = service;
  },
};