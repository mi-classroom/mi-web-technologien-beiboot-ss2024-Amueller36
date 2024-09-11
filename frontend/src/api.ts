import {type ApiResponse, type ProjectDataResponse, type UploadResponse } from './types';

const BACKEND_URL = import.meta.env.VITE_BACKEND_URL;


async function apiCall<T>(
    endpoint: string,
    method: string = 'GET',
    body?: any,
    headers?: HeadersInit
): Promise<ApiResponse<T>> {
    const url = getBackendUrlByEndpoint(endpoint);
    const options: RequestInit = {
        method,
        headers: {
            'Content-Type': 'application/json',
            ...headers,
        },
    };

    if (body) {
        options.body = JSON.stringify(body);
    }

    const response = await fetch(url, options);
    const data = await response.json();

    return { data, status: response.status };
}


export const api = {
    get: <T>(endpoint: string) => apiCall<T>(endpoint),
    post: <T>(endpoint: string, body: any) => apiCall<T>(endpoint, 'POST', body),
    put: <T>(endpoint: string, body: any) => apiCall<T>(endpoint, 'PUT', body),
    delete: <T>(endpoint: string) => apiCall<T>(endpoint, 'DELETE'),
};


export async function uploadFile(endpoint: string, formData: FormData, onProgress?: (progress: number) => void): Promise<ApiResponse<UploadResponse>> {
    const xhr = new XMLHttpRequest();

    const isSuccessfulStatus = (status : number) => {
        return status >= 200 && status < 300;
    }

    return new Promise((resolve, reject) => {
        xhr.open('POST', `${BACKEND_URL}${endpoint}`);

        xhr.upload.onprogress = (event) => {
            if (event.lengthComputable && onProgress) {
                const progress = Math.round((event.loaded / event.total) * 100);
                onProgress(progress);
            }
        };

        xhr.onload = () => {
            if (isSuccessfulStatus(xhr.status)) {
                resolve({ data: JSON.parse(xhr.responseText) as UploadResponse, status: xhr.status });
            } else {
                reject(new Error(`HTTP error! status: ${xhr.status}`));
            }
        };

        xhr.onerror = () => {
            reject(new Error('Network error'));
        };

        xhr.send(formData);
    });
}


export const endpoints = {
    projects: "/projects",
    specificProject: (videoId: string) => `/projects/${videoId}`,
    specificProject: (videoId: string) => `/projects/${videoId}`,
    upload: '/upload',
    createLongExposureImage: '/createLongExposureImage',
    frameThumbnail: (videoId: string, frameNumber: number) => 
      `/outputs/${videoId}/frames/ffout_thumbnail_${frameNumber.toString().padStart(4, '0')}.webp`,
    videoFile: (videoId: string, fileExtension: string) => 
      `/uploads/${videoId}.${fileExtension}`,
  };


export const getBackendUrlByEndpoint = (endpoint: string) => `${BACKEND_URL}${endpoint}`;
