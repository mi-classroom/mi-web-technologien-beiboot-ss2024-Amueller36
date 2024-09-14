import type {ApiResponse, UploadResponse } from './types';

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

    try {
        const response = await fetch(url, options);

        // Check if the response status indicates an error
        if (!response.ok) {
            // You can customize the error handling based on status codes
            const errorText = await response.text();
            throw new Error(`HTTP error! status: ${response.status}, message: ${errorText}`);
        }

        // Check Content-Type of response
        const contentType = response.headers.get('Content-Type');
        let data: T;

        if (contentType?.includes('application/json')) {
            // If response is JSON, parse it as JSON
            data = await response.json();
        } else if (contentType?.includes('text/plain')) {
            // If response is plain text, parse it as text
            const textData = await response.text();
            data = textData as unknown as T;
        } else {
            // Handle other response types if needed
            const fallbackData = await response.text();
            data = fallbackData as unknown as T;
        }

        return { data, status: response.status };
    } catch (error) {
        // Handle fetch errors or parsing errors
        // You can create a standardized error response
        console.error('API call failed:', error);
        throw error; // Re-throw the error after logging or handling it as needed
    }
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
    specificProject: (projectId: string) => `/projects/${projectId}`,
    createLongExposureImage: (projectId : string) => `/projects/${projectId}/createLongExposureImage`,
    frameThumbnail: (projectId: string, frameNumber: number) => 
      `/outputs/${projectId}/frames/ffout_thumbnail_${frameNumber.toString().padStart(4, '0')}.webp`,
    videoFile: (projectId: string, fileExtension: string) => 
      `/uploads/${projectId}.${fileExtension}`,
  };


export const getBackendUrlByEndpoint = (endpoint: string) => `${BACKEND_URL}${endpoint}`;
