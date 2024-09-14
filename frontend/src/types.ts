/* Api File Types */

export interface ApiResponse<T> {
    data: T;
    status: number;
}

/* Project Editor Types */

export interface Frame {
    src: string;
    frameNumber: number;
    time: string;
    weight: number;
}

export interface FrameToInclude {
    frame_number: number;
    frame_weight: number;
}

export interface CreateLongExposureImageRequest {
    frames_to_include: FrameToInclude[];
}

export interface ProjectDataResponse {
    fps: number;
    project_name: string;
    scale: string;
    video_file_extension: string;
    latest_long_exposure_image_name: string | null;
}

export interface UploadResponse {
    message: string;
    project_id: string;
}

/* Project Picker Types */

export interface ProjectsResponse {
    projects: Array<ProjectDetails>
}

export interface ProjectDetails {
    id: string,
    project_name: string,
    thumbnail_path: string,
}