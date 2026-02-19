use std::path::Path;

use crate::problem::vehicle_routing_problem::VehicleRoutingProblem;

use super::{cvrplib::CVRPLibParser, li_lim::LiLimParser, solomon::SolomonParser};

pub trait DatasetParser {
    fn parse(&self, content: &str) -> Result<VehicleRoutingProblem, anyhow::Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatasetFormat {
    Solomon,
    CvrpLib,
    LiLim,
}

/// Detect the dataset format by inspecting file contents.
fn detect_format(content: &str) -> Result<DatasetFormat, anyhow::Error> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // CVRPLIB files have "KEY : VALUE" header lines
        if trimmed.contains("NODE_COORD_SECTION")
            || trimmed.contains("DEMAND_SECTION")
            || (trimmed.contains(':')
                && trimmed.split_once(':').is_some_and(|(k, _)| {
                    matches!(
                        k.trim().to_uppercase().as_str(),
                        "NAME" | "TYPE" | "DIMENSION" | "CAPACITY" | "EDGE_WEIGHT_TYPE" | "COMMENT"
                    )
                }))
        {
            return Ok(DatasetFormat::CvrpLib);
        }

        // Solomon files have textual section headers
        if trimmed.starts_with("VEHICLE") || trimmed.starts_with("CUSTOMER") {
            return Ok(DatasetFormat::Solomon);
        }

        // First non-empty line is purely numeric — distinguish by data row column count
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let all_numeric = parts.iter().all(|p| p.parse::<f64>().is_ok());

        if all_numeric {
            // Check the second non-empty line to count data columns.
            // Li-Lim data rows have 9 columns, Solomon has 7.
            for next_line in content
                .lines()
                .skip_while(|l| l.trim().is_empty() || l.trim() == trimmed)
            {
                let next_trimmed = next_line.trim();
                if next_trimmed.is_empty() {
                    continue;
                }
                let next_parts: Vec<&str> = next_trimmed.split_whitespace().collect();
                if next_parts.len() == 9 {
                    return Ok(DatasetFormat::LiLim);
                }
                break;
            }

            return Ok(DatasetFormat::Solomon);
        }

        // Non-numeric first line that isn't a CVRPLIB header or Solomon keyword —
        // likely Solomon (instance name line like "C101")
        return Ok(DatasetFormat::Solomon);
    }

    Err(anyhow::anyhow!(
        "Could not detect dataset format: file is empty"
    ))
}

/// Read a file and parse it by auto-detecting the format.
pub fn parse_dataset<P: AsRef<Path>>(file: P) -> Result<VehicleRoutingProblem, anyhow::Error> {
    let content = std::fs::read_to_string(file)?;
    let format = detect_format(&content)?;
    match format {
        DatasetFormat::Solomon => SolomonParser.parse(&content),
        DatasetFormat::CvrpLib => CVRPLibParser.parse(&content),
        DatasetFormat::LiLim => LiLimParser.parse(&content),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_solomon() {
        let content = r#"C101

VEHICLE
NUMBER     CAPACITY
  25         200

CUSTOMER
CUST NO.  XCOORD.   YCOORD.    DEMAND   READY TIME  DUE DATE   SERVICE   TIME

    0      40         50          0          0       1236          0
    1      45         68         10        912        967         90
"#;
        assert_eq!(detect_format(content).unwrap(), DatasetFormat::Solomon);
    }

    #[test]
    fn test_detect_cvrplib() {
        let content = r#"NAME : A-n32-k5
COMMENT : (Augerat et al, No of trucks: 5, Optimal value: 784)
TYPE : CVRP
DIMENSION : 32
CAPACITY : 100
NODE_COORD_SECTION
 1 82 76
"#;
        assert_eq!(detect_format(content).unwrap(), DatasetFormat::CvrpLib);
    }

    #[test]
    fn test_detect_li_lim() {
        let content = "25\t200\t1\n\
                        0\t40\t50\t0\t0\t1236\t0\t0\t0\n\
                        1\t45\t68\t-10\t912\t967\t90\t11\t0\n\
                        2\t45\t70\t-20\t825\t870\t90\t6\t0\n";
        assert_eq!(detect_format(content).unwrap(), DatasetFormat::LiLim);
    }
}
